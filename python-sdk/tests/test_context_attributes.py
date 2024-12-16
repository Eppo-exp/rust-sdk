import pytest

from eppo_client.bandit import ContextAttributes


def test_init():
    ContextAttributes(numeric_attributes={"a": 12}, categorical_attributes={"b": "s"})


def test_init_unnamed():
    ContextAttributes({"a": 12}, {"b": "s"})


@pytest.mark.rust_only
def test_type_check():
    with pytest.raises(TypeError):
        ContextAttributes(
            numeric_attributes={"a": "s"}, categorical_attributes={"b": "s"}
        )


def test_bool_as_numeric():
    attrs = ContextAttributes(
        numeric_attributes={"true": True, "false": False}, categorical_attributes={}
    )
    assert attrs.numeric_attributes == {"true": 1.0, "false": 0.0}


def test_preserves_types_for_categorical_attributes():
    attrs = ContextAttributes(
        numeric_attributes={},
        categorical_attributes={"bool": True, "number": 42, "string": "hello"},
    )
    assert attrs.categorical_attributes == {
        "bool": True,
        "number": 42,
        "string": "hello",
    }


def test_empty():
    attrs = ContextAttributes.empty()


def test_from_dict():
    attrs = ContextAttributes.from_dict(
        {
            "numeric1": 1,
            "numeric2": 42.3,
            "categorical1": "string",
        }
    )
    assert attrs.numeric_attributes == {"numeric1": 1.0, "numeric2": 42.3}
    assert attrs.categorical_attributes == {
        "categorical1": "string",
    }


# `bool` is a subclass of `int` in Python, so it was incorrectly
# captured as numeric attribute:
# https://linear.app/eppo/issue/FF-3106/
@pytest.mark.rust_only
def test_from_dict_bool():
    attrs = ContextAttributes.from_dict(
        {
            "categorical": True,
        }
    )
    assert attrs.numeric_attributes == {}
    assert attrs.categorical_attributes == {
        "categorical": True,
    }


@pytest.mark.rust_only
def test_does_not_allow_bad_attributes():
    with pytest.raises(TypeError):
        attrs = ContextAttributes.from_dict({"custom": {"tested": True}})


# In Rust, context attributes live in Rust land and getter returns a
# copy of attributes.
@pytest.mark.rust_only
def test_attributes_are_frozen():
    attrs = ContextAttributes.from_dict({"cat": "string"})
    attrs.categorical_attributes["cat"] = "dog"
    assert attrs.categorical_attributes == {"cat": "string"}
