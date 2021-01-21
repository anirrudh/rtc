import pytest
import jupyter_rtc_automerge
import nbformat

def test_consume_cell():
    
    jupyter_rtc_automerge.automerge.consume_cell(
            cells=[nbformat.v4.new_code_cell(), nbformat.v4.new_code_cell()]
            )
    return

