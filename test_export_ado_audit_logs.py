import argparse
import base64
import io
import json
import unittest
from datetime import datetime, timezone
from unittest.mock import patch
from urllib.error import HTTPError

import export_ado_audit_logs as exporter


class FakeResponse:
    def __init__(self, payload):
        self.payload = payload

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        return False

    def read(self):
        return json.dumps(self.payload).encode("utf-8")


class AuthenticationTests(unittest.TestCase):
    def test_azure_devops_extension_pat_uses_basic_authentication(self):
        actual = exporter.get_authorization_header(
            {"AZURE_DEVOPS_EXT_PAT": "secret"}
        )
        expected = base64.b64encode(b":secret").decode("ascii")
        self.assertEqual(actual, f"Basic {expected}")

    def test_ado_pat_remains_supported(self):
        actual = exporter.get_authorization_header({"ADO_PAT": "legacy-secret"})
        expected = base64.b64encode(b":legacy-secret").decode("ascii")
        self.assertEqual(actual, f"Basic {expected}")

    def test_azure_devops_extension_pat_takes_precedence(self):
        actual = exporter.get_authorization_header(
            {
                "AZURE_DEVOPS_EXT_PAT": "preferred",
                "ADO_PAT": "fallback",
            }
        )
        expected = base64.b64encode(b":preferred").decode("ascii")
        self.assertEqual(actual, f"Basic {expected}")

    def test_access_token_uses_bearer_authentication(self):
        actual = exporter.get_authorization_header(
            {"ADO_ACCESS_TOKEN": "access-secret"}
        )
        self.assertEqual(actual, "Bearer access-secret")

    def test_rejects_ambiguous_credentials(self):
        with self.assertRaisesRegex(exporter.ExportError, "credential type"):
            exporter.get_authorization_header(
                {
                    "AZURE_DEVOPS_EXT_PAT": "pat",
                    "ADO_ACCESS_TOKEN": "token",
                }
            )


class DateTimeTests(unittest.TestCase):
    def test_parses_and_normalizes_rfc3339_timestamp(self):
        actual = exporter.parse_datetime("2026-07-30T20:30:00+08:00")
        self.assertEqual(
            actual, datetime(2026, 7, 30, 12, 30, tzinfo=timezone.utc)
        )

    def test_rejects_timestamp_without_offset(self):
        with self.assertRaises(argparse.ArgumentTypeError):
            exporter.parse_datetime("2026-07-30T12:30:00")

    def test_allows_zero_retries(self):
        self.assertEqual(exporter.nonnegative_int("0"), 0)


class ClientTests(unittest.TestCase):
    @patch("export_ado_audit_logs.urlopen")
    def test_accepts_unwrapped_query_result(self, mocked):
        mocked.return_value = FakeResponse(
            {
                "decoratedAuditLogEntries": [{"id": "direct"}],
                "continuationToken": None,
                "hasMore": False,
            }
        )
        client = exporter.AdoAuditClient("miniasp", "Bearer token", retries=0)

        pages = list(
            client.iter_pages(
                start_time=datetime(2026, 6, 1, tzinfo=timezone.utc),
                end_time=datetime(2026, 7, 1, tzinfo=timezone.utc),
                batch_size=200,
                skip_aggregation=True,
            )
        )

        self.assertEqual(pages, [[{"id": "direct"}]])

    @patch("export_ado_audit_logs.urlopen")
    def test_iterates_all_pages_and_url_encodes_continuation_token(self, mocked):
        mocked.side_effect = [
            FakeResponse(
                {
                    "value": {
                        "decoratedAuditLogEntries": [{"id": "first"}],
                        "continuationToken": "one;two/three",
                        "hasMore": True,
                    }
                }
            ),
            FakeResponse(
                {
                    "value": {
                        "decoratedAuditLogEntries": [{"id": "second"}],
                        "continuationToken": None,
                        "hasMore": False,
                    }
                }
            ),
        ]
        client = exporter.AdoAuditClient("miniasp", "Bearer token", retries=0)
        pages = list(
            client.iter_pages(
                start_time=datetime(2026, 7, 1, tzinfo=timezone.utc),
                end_time=datetime(2026, 7, 2, tzinfo=timezone.utc),
                batch_size=200,
                skip_aggregation=True,
            )
        )

        self.assertEqual(
            pages, [[{"id": "first"}], [{"id": "second"}]]
        )
        second_url = mocked.call_args_list[1].args[0].full_url
        self.assertIn("continuationToken=one%3Btwo%2Fthree", second_url)
        self.assertIn("skipAggregation=true", second_url)

    @patch("export_ado_audit_logs.urlopen")
    def test_retries_throttled_request(self, mocked):
        throttled = HTTPError(
            "https://example.invalid",
            429,
            "Too Many Requests",
            {"Retry-After": "0"},
            io.BytesIO(b'{"message":"slow down"}'),
        )
        mocked.side_effect = [
            throttled,
            FakeResponse(
                {
                    "value": {
                        "decoratedAuditLogEntries": [],
                        "continuationToken": None,
                        "hasMore": False,
                    }
                }
            ),
        ]
        sleeps = []
        client = exporter.AdoAuditClient(
            "miniasp",
            "Bearer token",
            retries=1,
            sleep=sleeps.append,
        )

        pages = list(
            client.iter_pages(
                start_time=datetime(2026, 7, 1, tzinfo=timezone.utc),
                end_time=datetime(2026, 7, 2, tzinfo=timezone.utc),
                batch_size=200,
                skip_aggregation=True,
            )
        )

        self.assertEqual(pages, [[]])
        self.assertEqual(sleeps, [0.0])

    def test_unexpected_shape_reports_keys_without_values(self):
        with self.assertRaisesRegex(
            exporter.ExportError,
            r"top-level keys: count, message; value type: missing",
        ):
            exporter.extract_query_result(
                {"count": 0, "message": "sensitive response detail"}
            )


class OutputTests(unittest.TestCase):
    def test_jsonl_preserves_nested_data(self):
        output = io.StringIO()
        count, pages = exporter.write_entries(
            output,
            "jsonl",
            iter([[{"id": "1", "data": {"ProjectName": "專案"}}]]),
            lambda _page, _count: None,
        )

        self.assertEqual(count, 1)
        self.assertEqual(pages, 1)
        self.assertEqual(
            json.loads(output.getvalue()),
            {"id": "1", "data": {"ProjectName": "專案"}},
        )

    def test_csv_serializes_data_and_unknown_fields(self):
        output = io.StringIO()
        exporter.write_entries(
            output,
            "csv",
            iter(
                [
                    [
                        {
                            "id": "1",
                            "data": {"ProjectName": "專案"},
                            "futureField": {"enabled": True},
                        }
                    ]
                ]
            ),
            lambda _page, _count: None,
        )

        text = output.getvalue()
        self.assertIn('"{""ProjectName"":""專案""}"', text)
        self.assertIn(
            '"{""futureField"":{""enabled"":true}}"',
            text,
        )


if __name__ == "__main__":
    unittest.main()
