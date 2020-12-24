import pytest

import charmap as rcharmap
import hypothesis.internal.charmap as pycharmap


@pytest.fixture(params=[rcharmap, pycharmap], ids=("rust", "python"))
def charmap(request):
    return request.param


@pytest.mark.benchmark(group="charmap")
def test_charmap(benchmark, charmap):
    benchmark(charmap.charmap)


@pytest.mark.benchmark(group="categories")
def test_categories(benchmark, charmap):
    benchmark(charmap.categories)


@pytest.mark.benchmark(group="as_general_categories")
def test_as_general_categories(benchmark, charmap):
    benchmark(charmap.as_general_categories, ["N"])


@pytest.mark.benchmark(group="query broad")
def test_query_broad(benchmark, charmap):
    benchmark(charmap.query)


@pytest.mark.benchmark(group="query narrow")
def test_query_narrow(benchmark, charmap):
    benchmark(charmap.query, None, ["Lu"], 0, max_codepoint=128, exclude_characters="N")


@pytest.mark.benchmark(group="query exclude & include")
def test_query_include_exclude(benchmark, charmap):
    benchmark(
        charmap.query,
        None,
        ["Lu"],
        0,
        max_codepoint=128,
        include_characters="0123456789",
        exclude_characters="QWERTYUIOP",
    )
