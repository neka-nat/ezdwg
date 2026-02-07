from ezdwg import document as document_module


def test_layer_color_overrides_for_ac1024_pattern() -> None:
    entity_style_map = {
        idx: (None, None, 896)
        for idx in range(1, 65)
    }
    entity_style_map[2000] = (None, None, 16)
    layer_color_map = {
        16: (0, 7),
        896: (0, 9),
        897: (0, 5),
    }

    overrides = document_module._layer_color_overrides(
        "AC1024", entity_style_map, layer_color_map
    )

    assert overrides == {896: (5, None), 16: (9, None)}


def test_layer_color_overrides_dynamic_handles() -> None:
    entity_style_map = {
        idx: (None, None, 1000)
        for idx in range(1, 51)
    }
    for idx in range(51, 61):
        entity_style_map[idx] = (None, None, 10)
    layer_color_map = {
        10: (0, 7),
        1000: (0, 9),
        2000: (0, 5),
    }

    overrides = document_module._layer_color_overrides(
        "AC1027", entity_style_map, layer_color_map
    )

    assert overrides == {1000: (5, None), 10: (9, None)}


def test_layer_color_overrides_skipped_if_897_is_used() -> None:
    entity_style_map = {
        1: (None, None, 896),
        2: (None, None, 897),
    }
    layer_color_map = {
        16: (0, 7),
        896: (0, 9),
        897: (0, 5),
    }

    overrides = document_module._layer_color_overrides(
        "AC1024", entity_style_map, layer_color_map
    )

    assert overrides == {}


def test_attach_entity_color_applies_arc_override() -> None:
    entity_style_map = {1: (None, None, 16)}
    layer_color_map = {
        16: (0, 7),
        896: (0, 9),
        897: (0, 5),
    }
    overrides = {896: (5, None), 16: (9, None)}

    dxf = document_module._attach_entity_color(
        1,
        {},
        entity_style_map,
        layer_color_map,
        overrides,
        dxftype="ARC",
    )

    assert dxf["resolved_color_index"] == 5


def test_line_supplementary_handles_detects_unique_long_segments() -> None:
    line_rows = [
        (1, 0.0, 0.0, 0.0, 10.0, 0.0, 0.0),
        (2, 10.0, 0.0, 0.0, 10.0, 1.0, 0.0),
        (3, 20.0, 0.0, 0.0, 30.0, 0.0, 0.0),
    ]
    entity_style_map = {
        1: (None, None, 896),
        2: (None, None, 896),
        3: (None, None, 896),
    }
    overrides = {896: (5, None), 16: (9, None)}

    result = document_module._line_supplementary_handles(
        line_rows, entity_style_map, overrides
    )

    assert result == {3}


def test_circle_supplementary_handles_detects_center_circle() -> None:
    circle_rows = [
        (10, 0.0, 0.0, 0.0, 3.0),
        (11, 0.0, 0.0, 0.0, 8.0),
        (12, 10.0, 0.0, 0.0, 2.0),
    ]
    entity_style_map = {
        10: (None, None, 896),
        11: (None, None, 896),
        12: (None, None, 896),
    }
    overrides = {896: (5, None), 16: (9, None)}

    result = document_module._circle_supplementary_handles(
        circle_rows, entity_style_map, overrides
    )

    assert result == {11}


def test_percentile_interpolates() -> None:
    value = document_module._percentile([1.0, 2.0, 5.0, 9.0], 0.5)
    assert value == 3.5
