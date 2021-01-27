use automerge_backend;
use automerge_frontend;
use automerge_protocol;
use bincode;
use log::{debug, info, LevelFilter};
use pyo3::conversion::FromPyObject;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyInt, PyList, PyString};
use pyo3::wrap_pyfunction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::vec;

/// This enum is important as it gives us all
/// the possible types for python dictionary
/// values.
///
/// TODO: Execution count can be a NoneType.
#[derive(Debug, Serialize, Deserialize, FromPyObject)]
enum nbCodeCell {
    id(String),
    cell_type(String),
    metadata(HashMap<String, String>),
    execution_count(i32),
    source(String),
    output(Vec<i32>),
}

#[derive(Debug, FromPyObject)]
struct nbModel<'a> {
    nbformat: i32,
    nbformat_minor: i32,
    metadata: Option<HashMap<String, &'a PyAny>>,
    cells: Option<Vec<HashMap<String, nbCodeCell>>>,
}

/// Takes a pythonic list with cells
/// from nbformat, stongly type them,
/// and then dump them.
fn _serialize_cells(cell_list: Vec<HashMap<String, nbCodeCell>>) -> Vec<u8> {
    info!("The cells that are being processed are: {:?}", cell_list);
    let bin_cells = bincode::serialize(&cell_list).unwrap();
    info!("After serialization: {:?}", bin_cells);
    return bin_cells;
}

/// Takes a notebook from python's nbformat
/// and then uses the `v4` version of the document
/// to let rust access the document using native
/// rust types.
///
/// Check logs for decompiled version.
#[pyfunction]
fn serialize_notebook(pynb: PyObject, py: Python) -> Vec<u8> {
    let nbformat_model: &PyDict = pynb.extract(py).unwrap();
    let mut nb = nbModel {
        nbformat: nbformat_model
            .get_item("nbformat")
            .unwrap()
            .extract()
            .unwrap(),
        nbformat_minor: nbformat_model
            .get_item("nbformat_minor")
            .unwrap()
            .extract()
            .unwrap(),
        metadata: nbformat_model
            .get_item("metadata")
            .unwrap()
            .extract()
            .unwrap(),
        cells: nbformat_model.get_item("cells").unwrap().extract().unwrap(),
    };
    info!("The nbmod from the dict: {:?}", nb);
    return _serialize_cells(nb.cells.unwrap());
}

#[pyfunction]
fn apply_changes(doc: Vec<u8>, change_bytes: Vec<u8>) -> Vec<u8> {
    let mut doc = automerge_backend::Backend::load(doc)
        .and_then(|back| Ok(back))
        .unwrap();
    let changes = automerge_backend::Change::from_bytes(change_bytes)
        .and_then(|c| Ok(c))
        .unwrap();
    doc.apply_changes(vec![changes])
        .and_then(|patch| Ok(patch))
        .unwrap();
    let data = doc.save().and_then(|data| Ok(data));
    return data.unwrap();
}

/// Initializes the Automerge Backend on the Rust side
///
/// Interal Method
/// Returns a new `automerge_backend` object.
fn _init_document(doc_id: &str, test_str: &str) -> automerge_backend::Backend {
    let mut doc = automerge_backend::Backend::init();
    let mut frontend = automerge_frontend::Frontend::new();
    let change = automerge_frontend::LocalChange::set(
        automerge_frontend::Path::root().key("docId"),
        automerge_frontend::Value::Primitive(automerge_protocol::ScalarValue::Str(
            doc_id.to_string(),
        )),
    );
    let change_request = frontend
        .change::<_, automerge_frontend::InvalidChangeRequest>(
            Some("set root object".into()),
            |frontend| {
                frontend.add_change(change)?;
                Ok(())
            },
        )
        .unwrap();
    doc.apply_local_change(change_request.unwrap()).unwrap().0;

    let changes = automerge_frontend::LocalChange::set(
        automerge_frontend::Path::root().key("textArea"),
        automerge_frontend::Value::Text(test_str.chars().collect()),
    );
    let change_request = frontend
        .change::<_, automerge_frontend::InvalidChangeRequest>(Some("".into()), |frontend| {
            frontend.add_change(changes)?;
            Ok(())
        })
        .unwrap();
    doc.apply_local_change(change_request.unwrap()).unwrap().0;
    return doc;
}

/// Processes the change that was recieved. The change will
/// come from python.
///
/// Internal Function
/// get_changes takes a byte-serialized
/// vector and returns the changes comp
#[pyfunction]
fn get_all_changes(doc: Vec<u8>) -> Vec<Vec<u8>> {
    let doc = automerge_backend::Backend::load(doc)
        .and_then(|back| Ok(back))
        .unwrap();
    let changes = doc.get_changes(&[]);
    let mut cb: Vec<Vec<u8>> = Vec::new();
    return cb;
}

/// `nbdoc` is shorthand for notebook document. This
/// is a special method which lets the notebook passed
/// into the function map to an automerge document.
///
/// Python Method
/// Returns a Vec<u8> to Python
#[pyfunction]
fn initialize_nbdoc(pynb: Vec<u8>) -> Vec<u8> {
    let hmp: Vec<HashMap<String, nbCodeCell>> = bincode::deserialize(&pynb).unwrap();
    info!("Deserialized, {:?}", hmp);
    let backend = _init_document("test-nbdoc", "test-function");
    let backend_data = backend.save().and_then(|data| Ok(data));
    return backend_data.unwrap();
}

pub fn init_submodule(module: &PyModule) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(serialize_notebook, module)?)?;
    module.add_function(wrap_pyfunction!(initialize_nbdoc, module)?)?;
    module.add_function(wrap_pyfunction!(get_all_changes, module)?)?;
    module.add_function(wrap_pyfunction!(apply_changes, module)?)?;
    Ok(())
}
