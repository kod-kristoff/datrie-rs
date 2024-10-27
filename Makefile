.DEFAULT: help

ifeq (${VIRTUAL_ENV},)
  VENV_NAME = .venv
  INVENV = uv run
else
  VENV_NAME = ${VIRTUAL_ENV}
  INVENV =
endif

.PHONY: help
help:
	@echo "Usage:"

${VENV_NAME}:
	test -d $@ || uv venv --seed $@

.PHONY: dev
dev: ${VENV_NAME} install-test
	${INVENV} maturin develop

install-test: tests/requirements.txt
	${INVENV} uv pip install -r $<

tests/requirements.txt: tests/requirements.in
	uv pip compile $< --output-file $@

.PHONY: test
test: dev
	${INVENV} pytest -vv tests
