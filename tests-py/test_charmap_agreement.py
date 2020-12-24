import sys

import charmap as rcm
import pytest
from hypothesis.internal import charmap as pcm
from hypothesis import strategies as st, given, assume


def test_charmap_equal():
    assert rcm.charmap() == pcm.charmap()


def test_categories_are_normalised():
    cm = pcm.charmap()
    # the categories are normalised, but categories with the same number of items may not have the same position
    for rcat, pcat in zip(rcm.categories(), pcm.categories()):
        # the number of items should be the same
        assert len(cm[rcat]) == len(cm[pcat])


@given(categories=st.lists(st.sampled_from(["L", "M", "N", "P", "S", "Z", "C"])))
def test_as_general_categories_equal(categories):
    assert set(rcm.as_general_categories(categories)) == set(pcm.as_general_categories(categories))


@given(categories=st.lists(st.characters(), min_size=1))
def test_as_general_categories_error(categories):
    assume(not set(categories).intersection({"L", "M", "N", "P", "S", "Z", "C"}))
    with pytest.raises(TypeError):
        rcm.as_general_categories(categories)
    with pytest.raises(TypeError):
        pcm.as_general_categories(categories)


@given(
    exclude_categories=st.lists(st.sampled_from(pcm.categories())),
    include_categories=st.lists(st.sampled_from(pcm.categories())),
    min_codepoint=st.integers(min_value=0, max_value=sys.maxunicode),
    max_codepoint=st.integers(min_value=0, max_value=sys.maxunicode),
    include_characters=st.text(),
    exclude_characters=st.text(),
)
def test_query_valid_agree(
    exclude_categories, include_categories, min_codepoint, max_codepoint, include_characters, exclude_characters
):
    min_codepoint, max_codepoint = sorted((min_codepoint, max_codepoint))
    assert rcm.query(
        exclude_categories, include_categories, min_codepoint, max_codepoint, include_characters, exclude_characters
    ) == pcm.query(
        exclude_categories, include_categories, min_codepoint, max_codepoint, include_characters, exclude_characters
    )


@given(
    exclude_categories=st.lists(st.characters()),
    include_categories=st.lists(st.characters()),
    min_codepoint=st.integers(),
    max_codepoint=st.integers(),
    include_characters=st.text(),
    exclude_characters=st.text(),
)
def test_query_invalid_agree(
    exclude_categories, include_categories, min_codepoint, max_codepoint, include_characters, exclude_characters
):
    rust_exception = None
    try:
        rcm.query(
            exclude_categories, include_categories, min_codepoint, max_codepoint, include_characters, exclude_characters
        )
    except Exception as rexc:
        rust_exception = rexc

    python_exception = None
    try:
        pcm.query(
            exclude_categories, include_categories, min_codepoint, max_codepoint, include_characters, exclude_characters
        )
    except Exception as pyexc:
        python_exception = pyexc

    if rust_exception and python_exception:
        if isinstance(python_exception, AssertionError) and "issubset of set object" in python_exception.args[0]:
            # This is guarded by `assert` statements in the Python version, but in the Rust one, there is an error
            # message that indicates the problem
            assert isinstance(rust_exception, TypeError)
            message = rust_exception.args[0]
            assert (
                "is not a valid Unicode category." in message
                or "Expected an iterable of valid Unicode categories" in message
            )
        else:
            assert rust_exception == python_exception
