// Python Wrappers
use pyo3::conversion::FromPyObject;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use pyo3::wrap_pyfunction;
// Automerge Libraries
use automerge_backend;
use automerge_frontend;
use automerge_protocol;
use log::{debug, info, LevelFilter};
use serde_json;
use simplelog::*;
use std::any::Any;
use std::boxed::Box;
use std::collections::HashMap;
use std::convert::From;

struct automergeObj {
    backend: automerge_backend::Backend,
    doc: automerge_frontend::Frontend,
}

fn default_backend() -> automerge_backend::Backend {
    let mut backend = automerge_backend::Backend::init();
    let mut doc = automerge_frontend::Frontend::new();

    let change = automerge_frontend::LocalChange::set(
        automerge_frontend::Path::root().key("docId"),
        automerge_frontend::Value::Primitive(automerge_protocol::ScalarValue::Str(
            "automerge-room".to_string(),
        )),
    );

    let change_request = doc
        .change::<_, automerge_frontend::InvalidChangeRequest>(
            Some("set root object".into()),
            |doc| {
                doc.add_change(change)?;
                Ok(())
            },
        )
        .unwrap();

    // let patch =
    backend
        .apply_local_change(change_request.unwrap())
        .unwrap()
        .0;

    // the textArea key contains the content of the client-side text area
    let change = automerge_frontend::LocalChange::set(
        automerge_frontend::Path::root().key("textArea"),
        automerge_frontend::Value::Text("Hello".chars().collect()),
    );

    let change_request = doc
        .change::<_, automerge_frontend::InvalidChangeRequest>(Some("".into()), |doc| {
            doc.add_change(change)?;
            Ok(())
        })
        .unwrap();

    // let patch =
    backend
        .apply_local_change(change_request.unwrap())
        .unwrap()
        .0;

    return backend;
}

#[pyfunction]
fn init_backend(py: Python, backend_data: PyObject) {
    let mut backend = automerge_backend::Backend::init();
    let mut doc = automerge_frontend::Frontend::new();
    return;
}

#[pyfunction]
fn new_backend(pyby: PyObject, py: Python) -> std::vec::Vec<u8> {
    let backend = default_backend();
    let backend_data = backend.save().and_then(|data| Ok(data));
    return backend_data.unwrap();
}

#[pyfunction]
fn apply_change(
    backend_data: std::vec::Vec<u8>,
    change_data: std::vec::Vec<u8>,
) -> std::vec::Vec<u8> {
    let mut backend = automerge_backend::Backend::load(backend_data)
        .and_then(|back| Ok(back))
        .unwrap();
    let change = automerge_backend::Change::from_bytes(change_data)
        .and_then(|c| Ok(c))
        .unwrap();
    backend
        .apply_changes(vec![change])
        .and_then(|patch| Ok(patch))
        .unwrap();
    let backend_data = backend.save().and_then(|data| Ok(data));
    return backend_data.unwrap();
}

fn clean_nb_string(nb_as_str: String) -> String {
    let mut nbon = &nb_as_str.trim_matches('\\');
    //let mut nbon = json!(nb_str);
    info!("After trimming: {:?} ", &nb_as_str.trim_matches('\\'));
    return nb_as_str;
}

fn serialize_json_string(nb_as_str: String) {
    info!(
        "The json string read from consume_notebook: {:?}",
        &nb_as_str
    );
    // let json1: serde_json::Value =
    //serde_json::from_str(&nb_as_str.trim_matches('\\')).expect("JSON was not well-formatted");
}

#[pyfunction]
fn consume_notebook(nb: PyObject, py: Python) {
    let res = nb.extract::<&PyDict>(py).unwrap();
    let res_str = res.to_string();
    let res_dict = res.into_iter();
    info!("The dictionary extracted was: {:?}", res);
    info!("The string extracted from nb: {:?}", res_str);
    let res1_str = clean_nb_string(res_str);
    serialize_json_string(res1_str);
}

#[pyfunction]
fn consume_cell(cells: PyObject, py: Python) {
    let cell_list = cells.extract::<&PyList>(py).unwrap();
    info!("The cells that were extracted: {:?}", cell_list);
}

#[pyfunction]
fn get_changes(backend_data: std::vec::Vec<u8>) -> std::vec::Vec<std::vec::Vec<u8>> {
    let backend = automerge_backend::Backend::load(backend_data)
        .and_then(|back| Ok(back))
        .unwrap();

    let changes = backend.get_changes(&[]);

    let mut changes_bytes: std::vec::Vec<std::vec::Vec<u8>> = std::vec::Vec::new();

    for c in changes.iter() {
        changes_bytes.push(c.bytes.clone());
    }

    return changes_bytes;
}

pub fn init_submodule(module: &PyModule) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(new_backend, module)?)?;
    module.add_function(wrap_pyfunction!(apply_change, module)?)?;
    module.add_function(wrap_pyfunction!(get_changes, module)?)?;
    module.add_function(wrap_pyfunction!(consume_notebook, module)?)?;
    module.add_function(wrap_pyfunction!(consume_cell, module)?)?;
    Ok(())
}
