import pytest
import jupyter_rtc_automerge 
import nbformat

def generate_test_notebook():
    gen_cells = [nbformat.v4.new_code_cell(execution_count=0) for i in range(5)] 
    test_nb = nbformat.v4.new_notebook()
    test_nb['cells'] = gen_cells
    return test_nb

def test_empty_notebook():
    test_nb = nbformat.v4.new_notebook()
    f = jupyter_rtc_automerge.nb.serialize_notebook(test_nb)
    return

def test_multicell_notebook():
    f = jupyter_rtc_automerge.nb.serialize_notebook(generate_test_notebook())
    return

def test_initalize_nbdoc():
   f = jupyter_rtc_automerge.nb.serialize_notebook(generate_test_notebook())
   jupyter_rtc_automerge.nb.initialize_nbdoc(pynb=f)
   return
 
