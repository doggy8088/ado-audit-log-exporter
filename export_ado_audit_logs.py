#!/usr/bin/env python3
"""Export Azure DevOps audit log entries through the REST API."""

from __future__ import annotations

import argparse
import base64
import csv
import json
import os
import sys
import tempfile
import time
from contextlib import contextmanager
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Callable, Iterator, TextIO
from urllib.error import HTTPError, URLError
from urllib.parse import quote, urlencode
from urllib.request import Request, urlopen


API_VERSION = "7.1-preview.1"
DEFAULT_ORGANIZATION = "miniasp"
DEFAULT_BATCH_SIZE = 200
DEFAULT_TIMEOUT_SECONDS = 30.0
DEFAULT_RETRIES = 4

CSV_FIELDS = [
    "id",
    "correlationId",
    "activityId",
    "timestamp",
    "actionId",
    "area",
    "category",
    "categoryDisplayName",
    "details",
    "actorCUID",
    "actorClientId",
    "actorUserId",
    "actorUPN",
    "actorDisplayName",
    "actorImageUrl",
    "authenticationMechanism",
    "ipAddress",
    "userAgent",
    "scopeType",
    "scopeDisplayName",
    "scopeId",
    "projectId",
    "projectName",
    "data",
    "extraFields",
]


class ExportError(RuntimeError):
    """An expected export failure with a user-facing message."""


def utc_now() -> datetime:
    return datetime.now(timezone.utc)


def format_datetime(value: datetime) -> str:
    """Return an Azure DevOps-compatible RFC 3339 UTC timestamp."""
    return value.astimezone(timezone.utc).isoformat(timespec="milliseconds").replace(
        "+00:00", "Z"
    )


def parse_datetime(value: str) -> datetime:
    """Parse an RFC 3339 timestamp and require an explicit UTC offset."""
    candidate = value.strip()
    if candidate.endswith(("Z", "z")):
        candidate = candidate[:-1] + "+00:00"
    try:
        parsed = datetime.fromisoformat(candidate)
    except ValueError as exc:
        raise argparse.ArgumentTypeError(
            f"invalid RFC 3339 timestamp: {value!r}"
        ) from exc
    if parsed.tzinfo is None:
        raise argparse.ArgumentTypeError(
            f"timestamp must include Z or a UTC offset: {value!r}"
        )
    return parsed.astimezone(timezone.utc)


def positive_int(value: str) -> int:
    try:
        parsed = int(value)
    except ValueError as exc:
        raise argparse.ArgumentTypeError(f"expected an integer: {value!r}") from exc
    if parsed <= 0:
        raise argparse.ArgumentTypeError("value must be greater than zero")
    return parsed


def nonnegative_int(value: str) -> int:
    try:
        parsed = int(value)
    except ValueError as exc:
        raise argparse.ArgumentTypeError(f"expected an integer: {value!r}") from exc
    if parsed < 0:
        raise argparse.ArgumentTypeError("value must be zero or greater")
    return parsed


def positive_float(value: str) -> float:
    try:
        parsed = float(value)
    except ValueError as exc:
        raise argparse.ArgumentTypeError(f"expected a number: {value!r}") from exc
    if parsed <= 0:
        raise argparse.ArgumentTypeError("value must be greater than zero")
    return parsed


def get_authorization_header(environ: dict[str, str]) -> str:
    """Build an authorization header without accepting secrets on the command line."""
    pat = environ.get("AZURE_DEVOPS_EXT_PAT", "") or environ.get("ADO_PAT", "")
    access_token = environ.get("ADO_ACCESS_TOKEN", "")
    if pat and access_token:
        raise ExportError(
            "set only one credential type: ADO_ACCESS_TOKEN or a PAT, not both"
        )
    if access_token:
        return f"Bearer {access_token}"
    if pat:
        encoded = base64.b64encode(f":{pat}".encode("utf-8")).decode("ascii")
        return f"Basic {encoded}"
    raise ExportError(
        "missing credentials; set AZURE_DEVOPS_EXT_PAT, ADO_ACCESS_TOKEN, "
        "or ADO_PAT in the environment"
    )


def read_http_error(error: HTTPError) -> str:
    try:
        body = error.read().decode("utf-8", errors="replace").strip()
    except OSError:
        return ""
    if not body:
        return ""
    try:
        payload = json.loads(body)
    except json.JSONDecodeError:
        return body[:1000]
    if isinstance(payload, dict):
        message = payload.get("message")
        if isinstance(message, str):
            return message
    return json.dumps(payload, ensure_ascii=False)[:1000]


def parse_retry_after(value: str | None, fallback: float) -> float:
    if value:
        try:
            return max(0.0, float(value))
        except ValueError:
            pass
    return fallback


def extract_query_result(payload: dict[str, Any]) -> dict[str, Any]:
    """Accept both Azure DevOps audit response shapes seen in the API."""
    if "decoratedAuditLogEntries" in payload:
        return payload

    wrapped = payload.get("value")
    if isinstance(wrapped, dict):
        return wrapped

    keys = ", ".join(sorted(str(key) for key in payload)) or "(none)"
    value_type = type(wrapped).__name__ if "value" in payload else "missing"
    raise ExportError(
        "unexpected API response shape: expected an AuditLogQueryResult at the "
        "top level or in 'value'; "
        f"top-level keys: {keys}; value type: {value_type}"
    )


class AdoAuditClient:
    def __init__(
        self,
        organization: str,
        authorization: str,
        *,
        timeout: float = DEFAULT_TIMEOUT_SECONDS,
        retries: int = DEFAULT_RETRIES,
        sleep: Callable[[float], None] = time.sleep,
    ) -> None:
        if not organization.strip():
            raise ExportError("organization cannot be empty")
        self.base_url = (
            "https://auditservice.dev.azure.com/"
            f"{quote(organization.strip(), safe='')}/_apis/audit/auditlog"
        )
        self.authorization = authorization
        self.timeout = timeout
        self.retries = retries
        self.sleep = sleep

    def _request_json(self, parameters: dict[str, str]) -> dict[str, Any]:
        url = f"{self.base_url}?{urlencode(parameters)}"
        request = Request(
            url,
            headers={
                "Accept": "application/json",
                "Authorization": self.authorization,
                "User-Agent": "ado-audit-log-exporter/1.0",
            },
            method="GET",
        )

        for attempt in range(self.retries + 1):
            try:
                with urlopen(request, timeout=self.timeout) as response:
                    body = response.read()
                payload = json.loads(body)
                if not isinstance(payload, dict):
                    raise ExportError(
                        "Azure DevOps returned a non-object JSON response"
                    )
                return payload
            except HTTPError as exc:
                retryable = exc.code == 429 or 500 <= exc.code <= 599
                if retryable and attempt < self.retries:
                    delay = parse_retry_after(
                        exc.headers.get("Retry-After"), min(2**attempt, 30)
                    )
                    self.sleep(delay)
                    continue
                detail = read_http_error(exc)
                suffix = f": {detail}" if detail else ""
                if exc.code == 401:
                    hint = (
                        "; verify the token and ensure a PAT includes the "
                        "vso.auditlog scope"
                    )
                elif exc.code == 403:
                    hint = (
                        "; the token owner needs the Azure DevOps "
                        "'View audit log' permission"
                    )
                else:
                    hint = ""
                raise ExportError(
                    f"Azure DevOps API returned HTTP {exc.code}{suffix}{hint}"
                ) from exc
            except URLError as exc:
                if attempt < self.retries:
                    self.sleep(min(2**attempt, 30))
                    continue
                raise ExportError(
                    f"could not reach Azure DevOps after {self.retries + 1} "
                    f"attempts: {exc.reason}"
                ) from exc
            except json.JSONDecodeError as exc:
                raise ExportError("Azure DevOps returned invalid JSON") from exc

        raise AssertionError("unreachable")

    def iter_pages(
        self,
        *,
        start_time: datetime,
        end_time: datetime,
        batch_size: int,
        skip_aggregation: bool,
    ) -> Iterator[list[dict[str, Any]]]:
        continuation_token: str | None = None
        seen_tokens: set[str] = set()

        while True:
            parameters = {
                "startTime": format_datetime(start_time),
                "endTime": format_datetime(end_time),
                "batchSize": str(batch_size),
                "skipAggregation": str(skip_aggregation).lower(),
                "api-version": API_VERSION,
            }
            if continuation_token:
                parameters["continuationToken"] = continuation_token

            payload = self._request_json(parameters)
            result = extract_query_result(payload)
            entries = result.get("decoratedAuditLogEntries")
            if not isinstance(entries, list) or not all(
                isinstance(entry, dict) for entry in entries
            ):
                raise ExportError(
                    "unexpected API response: "
                    "'decoratedAuditLogEntries' is not an array of objects"
                )
            yield entries

            has_more = result.get("hasMore")
            next_token = result.get("continuationToken")
            if not has_more:
                break
            if not isinstance(next_token, str) or not next_token:
                raise ExportError(
                    "unexpected API response: hasMore is true but no "
                    "continuationToken was returned"
                )
            if next_token in seen_tokens:
                raise ExportError(
                    "Azure DevOps repeated a continuation token; stopped to "
                    "avoid an infinite loop"
                )
            seen_tokens.add(next_token)
            continuation_token = next_token


def serialise_csv_value(value: Any) -> Any:
    if isinstance(value, (dict, list)):
        return json.dumps(value, ensure_ascii=False, separators=(",", ":"))
    if value is None:
        return ""
    return value


def to_csv_row(entry: dict[str, Any]) -> dict[str, Any]:
    known_fields = set(CSV_FIELDS) - {"extraFields"}
    row = {field: serialise_csv_value(entry.get(field)) for field in known_fields}
    extras = {
        key: value for key, value in entry.items() if key not in known_fields
    }
    row["extraFields"] = (
        json.dumps(extras, ensure_ascii=False, separators=(",", ":"))
        if extras
        else ""
    )
    return row


def write_entries(
    output: TextIO,
    output_format: str,
    pages: Iterator[list[dict[str, Any]]],
    progress: Callable[[int, int], None],
) -> tuple[int, int]:
    count = 0
    page_count = 0

    if output_format == "csv":
        writer = csv.DictWriter(output, fieldnames=CSV_FIELDS, extrasaction="ignore")
        writer.writeheader()
    elif output_format == "json":
        output.write("[")

    first_json_entry = True
    for page_count, entries in enumerate(pages, start=1):
        for entry in entries:
            if output_format == "json":
                if not first_json_entry:
                    output.write(",")
                json.dump(entry, output, ensure_ascii=False, separators=(",", ":"))
                first_json_entry = False
            elif output_format == "jsonl":
                json.dump(entry, output, ensure_ascii=False, separators=(",", ":"))
                output.write("\n")
            else:
                writer.writerow(to_csv_row(entry))
            count += 1
        progress(page_count, count)

    if output_format == "json":
        output.write("]\n")
    output.flush()
    return count, page_count


@contextmanager
def output_stream(
    destination: str, *, overwrite: bool
) -> Iterator[tuple[TextIO, Path | None]]:
    if destination == "-":
        yield sys.stdout, None
        return

    path = Path(destination).expanduser().resolve()
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists() and not overwrite:
        raise ExportError(
            f"output file already exists: {path}; pass --overwrite to replace it"
        )

    temporary: Path | None = None
    stream: TextIO | None = None
    try:
        file_descriptor, temporary_name = tempfile.mkstemp(
            prefix=f".{path.name}.", suffix=".tmp", dir=path.parent
        )
        temporary = Path(temporary_name)
        stream = os.fdopen(file_descriptor, "w", encoding="utf-8", newline="")
        yield stream, path
        stream.close()
        stream = None
        os.replace(temporary, path)
        temporary = None
    finally:
        if stream is not None:
            stream.close()
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def build_parser(now: datetime | None = None) -> argparse.ArgumentParser:
    reference_time = now or utc_now()
    default_start = reference_time - timedelta(days=90)

    parser = argparse.ArgumentParser(
        description=(
            "Export Azure DevOps audit logs through the REST API. Credentials "
            "are read from AZURE_DEVOPS_EXT_PAT, ADO_ACCESS_TOKEN, or ADO_PAT."
        )
    )
    parser.add_argument(
        "--organization",
        default=DEFAULT_ORGANIZATION,
        help=f"Azure DevOps organization name (default: {DEFAULT_ORGANIZATION})",
    )
    parser.add_argument(
        "--start-time",
        type=parse_datetime,
        default=default_start,
        metavar="RFC3339",
        help="inclusive start timestamp (default: 90 days ago)",
    )
    parser.add_argument(
        "--end-time",
        type=parse_datetime,
        default=reference_time,
        metavar="RFC3339",
        help="inclusive end timestamp (default: now)",
    )
    parser.add_argument(
        "--format",
        choices=("json", "jsonl", "csv"),
        default="jsonl",
        help="export format (default: jsonl)",
    )
    parser.add_argument(
        "--output",
        default="-",
        metavar="PATH",
        help="output file, or - for standard output (default: -)",
    )
    parser.add_argument(
        "--batch-size",
        type=positive_int,
        default=DEFAULT_BATCH_SIZE,
        help=f"entries requested per API page (default: {DEFAULT_BATCH_SIZE})",
    )
    parser.add_argument(
        "--aggregate-access-log",
        action="store_true",
        help=(
            "allow Azure DevOps to aggregate AuditLog.AccessLog events; by "
            "default each access event is exported separately"
        ),
    )
    parser.add_argument(
        "--timeout",
        type=positive_float,
        default=DEFAULT_TIMEOUT_SECONDS,
        metavar="SECONDS",
        help=f"HTTP request timeout (default: {DEFAULT_TIMEOUT_SECONDS:g})",
    )
    parser.add_argument(
        "--retries",
        type=nonnegative_int,
        default=DEFAULT_RETRIES,
        help=(
            "retry count for HTTP 429/5xx and network errors "
            f"(default: {DEFAULT_RETRIES})"
        ),
    )
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="replace the output file if it already exists",
    )
    return parser


def run(arguments: argparse.Namespace, environ: dict[str, str]) -> int:
    if arguments.start_time > arguments.end_time:
        raise ExportError("--start-time must not be later than --end-time")

    authorization = get_authorization_header(environ)
    client = AdoAuditClient(
        arguments.organization,
        authorization,
        timeout=arguments.timeout,
        retries=arguments.retries,
    )
    pages = client.iter_pages(
        start_time=arguments.start_time,
        end_time=arguments.end_time,
        batch_size=arguments.batch_size,
        skip_aggregation=not arguments.aggregate_access_log,
    )

    def progress(page_count: int, entry_count: int) -> None:
        print(
            f"Fetched {entry_count} entries in {page_count} page(s)...",
            file=sys.stderr,
        )

    with output_stream(arguments.output, overwrite=arguments.overwrite) as (
        output,
        output_path,
    ):
        entry_count, page_count = write_entries(
            output, arguments.format, pages, progress
        )

    destination = str(output_path) if output_path else "standard output"
    print(
        f"Exported {entry_count} entries from {page_count} page(s) to {destination}.",
        file=sys.stderr,
    )
    return 0


def main() -> int:
    parser = build_parser()
    arguments = parser.parse_args()
    try:
        return run(arguments, dict(os.environ))
    except ExportError as exc:
        parser.exit(1, f"error: {exc}\n")
    except BrokenPipeError:
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
