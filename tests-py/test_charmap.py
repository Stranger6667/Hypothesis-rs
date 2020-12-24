# This file is part of Hypothesis, which may be found at
# https://github.com/HypothesisWorks/hypothesis/
#
# Most of this work is copyright (C) 2013-2020 David R. MacIver
# (david@drmaciver.com), but it contains contributions by others. See
# CONTRIBUTING.rst for a full list of people who may hold copyright, and
# consult the git log if you need to determine who owns an individual
# contribution.
#
# This Source Code Form is subject to the terms of the Mozilla Public License,
# v. 2.0. If a copy of the MPL was not distributed with this file, You can
# obtain one at https://mozilla.org/MPL/2.0/.
#
# END HEADER
import sys
import unicodedata

import charmap as cm
from hypothesis import assume, given, strategies as st


def assert_valid_range_list(ls):
    for u, v in ls:
        assert u <= v
    for i in range(len(ls) - 1):
        assert ls[i] <= ls[i + 1]
        assert ls[i][-1] < ls[i + 1][0]


def test_charmap_contains_all_unicode():
    n = 0
    for vs in cm.charmap().values():
        for u, v in vs:
            n += v - u + 1
    assert n == sys.maxunicode + 1


def test_charmap_has_right_categories():
    for cat, intervals in cm.charmap().items():
        for u, v in intervals:
            for i in range(u, v + 1):
                real = unicodedata.category(chr(i))
                assert real == cat, "%d is %s but reported in %s" % (i, real, cat)


@given(
    st.sets(st.sampled_from(cm.categories())),
    st.sets(st.sampled_from(cm.categories())) | st.none(),
)
def test_query_matches_categories(exclude, include):
    values = cm.query(exclude, include)
    assert_valid_range_list(values)
    for u, v in values:
        for i in (u, v, (u + v) // 2):
            cat = unicodedata.category(chr(i))
            if include is not None:
                assert cat in include
            assert cat not in exclude


@given(
    st.sets(st.sampled_from(cm.categories())),
    st.sets(st.sampled_from(cm.categories())) | st.none(),
    st.integers(0, sys.maxunicode),
    st.integers(0, sys.maxunicode),
)
def test_query_matches_categories_codepoints(exclude, include, m1, m2):
    m1, m2 = sorted((m1, m2))
    values = cm.query(exclude, include, min_codepoint=m1, max_codepoint=m2)
    assert_valid_range_list(values)
    for u, v in values:
        assert m1 <= u
        assert v <= m2


@given(st.sampled_from(cm.categories()), st.integers(0, sys.maxunicode))
def test_exclude_only_excludes_from_that_category(cat, i):
    c = chr(i)
    assume(unicodedata.category(c) != cat)
    intervals = cm.query(exclude_categories=(cat,))
    assert any(a <= i <= b for a, b in intervals)


def test_exclude_characters_are_included_in_key():
    assert cm.query() != cm.query(exclude_characters="0")
