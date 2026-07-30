SHELL := /bin/sh
.DEFAULT_GOAL := help

PYTHON ?= python3
ORGANIZATION ?= miniasp
FORMAT ?= jsonl
OUTPUT ?= ado-audit.$(FORMAT)
START_TIME ?=
END_TIME ?=
BATCH_SIZE ?= 200
TIMEOUT ?= 30
RETRIES ?= 4
AGGREGATE_ACCESS_LOG ?= 0
OVERWRITE ?= 0

AZURE_DEVOPS_RESOURCE_ID := 499b84ac-1321-427f-aa17-267ca6975798
EXPORTER := export_ado_audit_logs.py

.PHONY: help export export-entra export-json export-jsonl export-csv test check

help:
	@printf '%s\n' \
		'Azure DevOps 稽核記錄匯出工具' \
		'' \
		'目標：' \
		'  make export         使用 AZURE_DEVOPS_EXT_PAT 或存取權杖匯出' \
		'  make export-entra   透過已登入的 Azure CLI 取得暫時權杖後匯出' \
		'  make export-json    匯出 JSON' \
		'  make export-jsonl   匯出 JSON Lines' \
		'  make export-csv     匯出 CSV' \
		'  make test           執行單元測試' \
		'  make check          執行語法檢查與單元測試' \
		'' \
		'可覆寫變數：' \
		'  ORGANIZATION=miniasp' \
		'  FORMAT=jsonl' \
		'  OUTPUT=ado-audit.jsonl' \
		'  START_TIME=2026-07-01T00:00:00Z' \
		'  END_TIME=2026-07-30T23:59:59Z' \
		'  BATCH_SIZE=200 TIMEOUT=30 RETRIES=4' \
		'  AGGREGATE_ACCESS_LOG=1 OVERWRITE=1'

export:
	@set -eu; \
	if [ -z "$${AZURE_DEVOPS_EXT_PAT:-}" ] && \
	   [ -z "$${ADO_ACCESS_TOKEN:-}" ] && \
	   [ -z "$${ADO_PAT:-}" ]; then \
		printf '%s\n' \
			'錯誤：請設定 AZURE_DEVOPS_EXT_PAT、ADO_ACCESS_TOKEN 或 ADO_PAT，或執行 make export-entra。' >&2; \
		exit 1; \
	fi; \
	set -- \
		--organization "$(ORGANIZATION)" \
		--format "$(FORMAT)" \
		--output "$(OUTPUT)" \
		--batch-size "$(BATCH_SIZE)" \
		--timeout "$(TIMEOUT)" \
		--retries "$(RETRIES)"; \
	if [ -n "$(START_TIME)" ]; then \
		set -- "$$@" --start-time "$(START_TIME)"; \
	fi; \
	if [ -n "$(END_TIME)" ]; then \
		set -- "$$@" --end-time "$(END_TIME)"; \
	fi; \
	if [ "$(AGGREGATE_ACCESS_LOG)" = "1" ]; then \
		set -- "$$@" --aggregate-access-log; \
	fi; \
	if [ "$(OVERWRITE)" = "1" ]; then \
		set -- "$$@" --overwrite; \
	fi; \
	exec "$(PYTHON)" "$(EXPORTER)" "$$@"

export-entra:
	@set -eu; \
	if ! command -v az >/dev/null 2>&1; then \
		printf '%s\n' '錯誤：找不到 Azure CLI 的 az 指令。' >&2; \
		exit 1; \
	fi; \
	access_token="$$( \
		az account get-access-token \
			--resource "$(AZURE_DEVOPS_RESOURCE_ID)" \
			--query accessToken \
			--output tsv \
	)"; \
	if [ -z "$$access_token" ]; then \
		printf '%s\n' '錯誤：Azure CLI 未傳回存取權杖。' >&2; \
		exit 1; \
	fi; \
	unset AZURE_DEVOPS_EXT_PAT ADO_PAT; \
	export ADO_ACCESS_TOKEN="$$access_token"; \
	exec "$(MAKE)" --no-print-directory export

export-json: FORMAT = json
export-json: export

export-jsonl: FORMAT = jsonl
export-jsonl: export

export-csv: FORMAT = csv
export-csv: export

test:
	"$(PYTHON)" -m unittest -v

check:
	"$(PYTHON)" -m compileall -q "$(EXPORTER)" test_export_ado_audit_logs.py
	"$(PYTHON)" -m unittest -v
