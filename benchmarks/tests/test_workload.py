import json
from pathlib import Path

import pytest

from benchmarks.run import load_workload


WORKLOAD = Path(__file__).parents[1] / "workload-v1.json"


def test_carga_versionada_define_tres_comparadores_multiproceso() -> None:
    workload = load_workload(WORKLOAD)

    assert workload["version"] == "v1"
    assert workload["worker_count"] == 2
    assert workload["named_databases"] == ["ventas", "inventario"]
    assert workload["query_operations_per_worker"] > 0
    assert workload["write_operations_per_worker"] > 0


def test_carga_rechaza_nombres_de_base_duplicados(tmp_path: Path) -> None:
    workload = json.loads(WORKLOAD.read_text(encoding="utf-8"))
    workload["named_databases"] = ["ventas", "ventas"]
    invalid = tmp_path / "invalida.json"
    invalid.write_text(json.dumps(workload), encoding="utf-8")

    with pytest.raises(ValueError, match="únicos"):
        load_workload(invalid)
