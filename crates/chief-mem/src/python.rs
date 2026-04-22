use std::path::PathBuf;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

use crate::{ChiefMem, Edge, EdgeKind, Horizon, Node, NodeType};

#[pyclass(name = "ChiefMem")]
pub struct PyChiefMem {
    inner: ChiefMem,
}

#[pymethods]
impl PyChiefMem {
    #[new]
    fn new(path: PathBuf) -> PyResult<Self> {
        Ok(Self {
            inner: ChiefMem::open(&path).map_err(to_runtime_error)?,
        })
    }

    fn put_node(
        &self,
        node_type: &str,
        horizon: &str,
        confidence: f64,
        source: String,
        body: String,
        blob_ref: Option<String>,
        meta_json: Option<String>,
    ) -> PyResult<String> {
        let node_type = node_type
            .parse::<NodeType>()
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        let horizon = horizon
            .parse::<Horizon>()
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        let meta_json = meta_json
            .map(|raw| serde_json::from_str(&raw))
            .transpose()
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        self.inner
            .put_node(Node {
                node_type,
                horizon,
                confidence,
                source,
                body,
                blob_ref,
                meta_json,
            })
            .map_err(to_runtime_error)
    }

    fn put_edge(
        &self,
        from: String,
        to: String,
        kind: &str,
        weight: f64,
        created_by: String,
    ) -> PyResult<()> {
        let kind = kind
            .parse::<EdgeKind>()
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        self.inner
            .put_edge(Edge {
                from,
                to,
                kind,
                weight,
                created_by,
            })
            .map_err(to_runtime_error)
    }

    fn put_blob(&self, data: &[u8]) -> PyResult<String> {
        self.inner.put_blob(data).map_err(to_runtime_error)
    }

    fn query(
        &self,
        scope: Option<Vec<String>>,
        horizon: Option<Vec<String>>,
        text: &str,
        k: usize,
    ) -> PyResult<String> {
        let scope = parse_node_types(scope)?;
        let horizon = parse_horizons(horizon)?;
        let results = self
            .inner
            .query(scope.as_deref(), horizon.as_deref(), text, k)
            .map_err(to_runtime_error)?;
        serde_json::to_string(&results).map_err(to_runtime_error)
    }
}

#[pymodule]
fn chief_mem(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyChiefMem>()?;
    Ok(())
}

fn parse_node_types(values: Option<Vec<String>>) -> PyResult<Option<Vec<NodeType>>> {
    values
        .map(|values| {
            values
                .iter()
                .map(|value| {
                    value
                        .parse::<NodeType>()
                        .map_err(|err| PyValueError::new_err(err.to_string()))
                })
                .collect()
        })
        .transpose()
}

fn parse_horizons(values: Option<Vec<String>>) -> PyResult<Option<Vec<Horizon>>> {
    values
        .map(|values| {
            values
                .iter()
                .map(|value| {
                    value
                        .parse::<Horizon>()
                        .map_err(|err| PyValueError::new_err(err.to_string()))
                })
                .collect()
        })
        .transpose()
}

fn to_runtime_error<E: std::fmt::Display>(err: E) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}
