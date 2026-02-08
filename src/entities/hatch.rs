use crate::bit::{BitReader, Endian};
use crate::core::error::{DwgError, ErrorKind};
use crate::core::result::Result;
use crate::entities::common::{
    parse_common_entity_handles, parse_common_entity_header, parse_common_entity_header_r2007,
    parse_common_entity_header_r2010, parse_common_entity_header_r2013,
    parse_common_entity_layer_handle, CommonEntityHeader,
};

#[derive(Debug, Clone)]
pub struct HatchPath {
    pub closed: bool,
    pub points: Vec<(f64, f64)>,
}

#[derive(Debug, Clone)]
pub struct HatchEntity {
    pub handle: u64,
    pub color_index: Option<u16>,
    pub true_color: Option<u32>,
    pub layer_handle: u64,
    pub name: String,
    pub solid_fill: bool,
    pub associative: bool,
    pub elevation: f64,
    pub extrusion: (f64, f64, f64),
    pub paths: Vec<HatchPath>,
}

pub fn decode_hatch(reader: &mut BitReader<'_>) -> Result<HatchEntity> {
    let header = parse_common_entity_header(reader)?;
    decode_hatch_with_header(reader, header, false, false, false)
}

pub fn decode_hatch_r2004(reader: &mut BitReader<'_>) -> Result<HatchEntity> {
    let header = parse_common_entity_header(reader)?;
    decode_hatch_with_header(reader, header, false, false, true)
}

pub fn decode_hatch_r2007(reader: &mut BitReader<'_>) -> Result<HatchEntity> {
    let header = parse_common_entity_header_r2007(reader)?;
    decode_hatch_with_header(reader, header, true, true, true)
}

pub fn decode_hatch_r2010(
    reader: &mut BitReader<'_>,
    object_data_end_bit: u32,
    object_handle: u64,
) -> Result<HatchEntity> {
    let mut header = parse_common_entity_header_r2010(reader, object_data_end_bit)?;
    header.handle = object_handle;
    decode_hatch_with_header(reader, header, true, true, true)
}

pub fn decode_hatch_r2013(
    reader: &mut BitReader<'_>,
    object_data_end_bit: u32,
    object_handle: u64,
) -> Result<HatchEntity> {
    let mut header = parse_common_entity_header_r2013(reader, object_data_end_bit)?;
    header.handle = object_handle;
    decode_hatch_with_header(reader, header, true, true, true)
}

fn decode_hatch_with_header(
    reader: &mut BitReader<'_>,
    header: CommonEntityHeader,
    allow_handle_decode_failure: bool,
    r2007_layer_only: bool,
    has_gradient_payload: bool,
) -> Result<HatchEntity> {
    if has_gradient_payload {
        skip_gradient_payload(reader)?;
    }

    let elevation = reader.read_bd()?;
    let extrusion = reader.read_3bd()?;
    let name = reader.read_tv()?;
    let solid_fill = reader.read_b()? != 0;
    let associative = reader.read_b()? != 0;

    let num_paths = bounded_count(reader.read_bl()?, "hatch paths")?;
    let mut paths = Vec::with_capacity(num_paths);
    let mut any_path_uses_pixel_size = false;

    for _ in 0..num_paths {
        let path_flag = reader.read_bl()?;
        any_path_uses_pixel_size |= (path_flag & 0x04) != 0;

        if (path_flag & 0x02) == 0 {
            let num_segments = bounded_count(reader.read_bl()?, "hatch edge path segments")?;
            let mut path_points: Vec<(f64, f64)> = Vec::new();
            for _ in 0..num_segments {
                let segment_type = reader.read_rc()?;
                match segment_type {
                    1 => {
                        let start = read_point2rd(reader)?;
                        let end = read_point2rd(reader)?;
                        append_segment_points(&mut path_points, &[start, end]);
                    }
                    2 => {
                        let center = read_point2rd(reader)?;
                        let radius = reader.read_bd()?;
                        let start_angle = reader.read_bd()?;
                        let end_angle = reader.read_bd()?;
                        let is_ccw = reader.read_b()? != 0;
                        let segment =
                            circular_arc_points(center, radius, start_angle, end_angle, is_ccw, 64);
                        append_segment_points(&mut path_points, &segment);
                    }
                    3 => {
                        let center = read_point2rd(reader)?;
                        let major_endpoint = read_point2rd(reader)?;
                        let ratio = reader.read_bd()?;
                        let start_angle = reader.read_bd()?;
                        let end_angle = reader.read_bd()?;
                        let is_ccw = reader.read_b()? != 0;
                        let segment = elliptical_arc_points(
                            center,
                            major_endpoint,
                            ratio,
                            start_angle,
                            end_angle,
                            is_ccw,
                            96,
                        );
                        append_segment_points(&mut path_points, &segment);
                    }
                    4 => {
                        return Err(DwgError::new(
                            ErrorKind::NotImplemented,
                            "HATCH spline edge is not supported yet",
                        ));
                    }
                    _ => {
                        return Err(DwgError::new(
                            ErrorKind::Format,
                            format!("unsupported HATCH edge segment type: {segment_type}"),
                        ));
                    }
                }
            }
            let _num_boundary_obj_handles = reader.read_bl()?;
            close_path_if_needed(&mut path_points);
            paths.push(HatchPath {
                closed: true,
                points: path_points,
            });
            continue;
        }

        let bulges_present = reader.read_b()? != 0;
        let closed = reader.read_b()? != 0;
        let num_vertices = bounded_count(reader.read_bl()?, "hatch polyline vertices")?;
        let mut vertices: Vec<(f64, f64)> = Vec::with_capacity(num_vertices);
        let mut bulges: Vec<f64> = Vec::with_capacity(num_vertices);
        for _ in 0..num_vertices {
            vertices.push(read_point2rd(reader)?);
            if bulges_present {
                bulges.push(reader.read_bd()?);
            }
        }
        let _num_boundary_obj_handles = reader.read_bl()?;

        let mut points = if bulges_present {
            polyline_with_bulges_points(&vertices, &bulges, closed, 64)
        } else {
            vertices
        };
        if closed {
            close_path_if_needed(&mut points);
        }
        paths.push(HatchPath { closed, points });
    }

    if let Err(err) = skip_hatch_definition_payload(reader, solid_fill, any_path_uses_pixel_size) {
        if !matches!(
            err.kind,
            ErrorKind::Format | ErrorKind::Decode | ErrorKind::Io
        ) {
            return Err(err);
        }
    }

    // Handles are stored in the handle stream at obj_size bit offset.
    reader.set_bit_pos(header.obj_size);
    let layer_handle = match if r2007_layer_only {
        parse_common_entity_layer_handle(reader, &header)
    } else {
        parse_common_entity_handles(reader, &header).map(|common_handles| common_handles.layer)
    } {
        Ok(layer_handle) => layer_handle,
        Err(err)
            if allow_handle_decode_failure
                && matches!(
                    err.kind,
                    ErrorKind::Format | ErrorKind::Decode | ErrorKind::Io
                ) =>
        {
            0
        }
        Err(err) => return Err(err),
    };

    Ok(HatchEntity {
        handle: header.handle,
        color_index: header.color.index,
        true_color: header.color.true_color,
        layer_handle,
        name,
        solid_fill,
        associative,
        elevation,
        extrusion,
        paths,
    })
}

fn skip_gradient_payload(reader: &mut BitReader<'_>) -> Result<()> {
    let _is_gradient = reader.read_bl()?;
    let _reserved = reader.read_bl()?;
    let _gradient_angle = reader.read_bd()?;
    let _gradient_shift = reader.read_bd()?;
    let _single_color = reader.read_bl()?;
    let _gradient_tint = reader.read_bd()?;
    let num_colors = bounded_count(reader.read_bl()?, "hatch gradient colors")?;
    for _ in 0..num_colors {
        let _unknown_double = reader.read_bd()?;
        let _unknown_short = reader.read_bs()?;
        let _rgb_color = reader.read_bl()?;
        let _ignored_color_byte = reader.read_rc()?;
    }
    let _gradient_name = reader.read_tv()?;
    Ok(())
}

fn skip_hatch_definition_payload(
    reader: &mut BitReader<'_>,
    solid_fill: bool,
    any_path_uses_pixel_size: bool,
) -> Result<()> {
    let _style = reader.read_bs()?;
    let _pattern_type = reader.read_bs()?;

    if !solid_fill {
        let _pattern_angle = reader.read_bd()?;
        let _pattern_scale = reader.read_bd()?;
        let _double_hatch = reader.read_b()?;
        let num_def_lines =
            bounded_count(reader.read_bs()? as u32, "hatch pattern definition lines")?;
        for _ in 0..num_def_lines {
            let _line_angle = reader.read_bd()?;
            let _line_origin = (reader.read_bd()?, reader.read_bd()?);
            let _line_offset = (reader.read_bd()?, reader.read_bd()?);
            let num_dashes = bounded_count(reader.read_bs()? as u32, "hatch pattern dashes")?;
            for _ in 0..num_dashes {
                let _dash_length = reader.read_bd()?;
            }
        }
    }

    let num_seed_points = if any_path_uses_pixel_size {
        let _pixel_size = reader.read_bd()?;
        bounded_count(reader.read_bl()?, "hatch seed points")?
    } else {
        0usize
    };
    for _ in 0..num_seed_points {
        let _seed = read_point2rd(reader)?;
    }
    Ok(())
}

fn read_point2rd(reader: &mut BitReader<'_>) -> Result<(f64, f64)> {
    Ok((
        reader.read_rd(Endian::Little)?,
        reader.read_rd(Endian::Little)?,
    ))
}

fn append_segment_points(points: &mut Vec<(f64, f64)>, segment: &[(f64, f64)]) {
    if segment.is_empty() {
        return;
    }
    if points.is_empty() {
        points.extend_from_slice(segment);
        return;
    }
    let mut start = 0usize;
    if points_equal_2d(*points.last().unwrap(), segment[0]) {
        start = 1;
    }
    points.extend_from_slice(&segment[start..]);
}

fn close_path_if_needed(points: &mut Vec<(f64, f64)>) {
    if points.len() <= 1 {
        return;
    }
    let first = points[0];
    let last = *points.last().unwrap();
    if !points_equal_2d(first, last) {
        points.push(first);
    }
}

fn circular_arc_points(
    center: (f64, f64),
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    is_ccw: bool,
    arc_segments: usize,
) -> Vec<(f64, f64)> {
    if radius.abs() <= 1.0e-12 {
        return vec![];
    }
    let sweep = normalized_sweep(start_angle, end_angle, is_ccw);
    let segs = ((sweep.abs() / std::f64::consts::TAU) * (arc_segments.max(8) as f64)).ceil();
    let segments = segs.max(2.0) as usize;
    let mut out = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let t = (i as f64) / (segments as f64);
        let angle = start_angle + sweep * t;
        out.push((
            center.0 + radius * angle.cos(),
            center.1 + radius * angle.sin(),
        ));
    }
    out
}

fn elliptical_arc_points(
    center: (f64, f64),
    major_endpoint: (f64, f64),
    ratio: f64,
    start_angle: f64,
    end_angle: f64,
    is_ccw: bool,
    arc_segments: usize,
) -> Vec<(f64, f64)> {
    let mx = major_endpoint.0;
    let my = major_endpoint.1;
    if mx.abs() <= 1.0e-12 && my.abs() <= 1.0e-12 {
        return vec![];
    }
    let vx = -my * ratio;
    let vy = mx * ratio;
    let sweep = normalized_sweep(start_angle, end_angle, is_ccw);
    let segs = ((sweep.abs() / std::f64::consts::TAU) * (arc_segments.max(16) as f64)).ceil();
    let segments = segs.max(4.0) as usize;
    let mut out = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let t = (i as f64) / (segments as f64);
        let angle = start_angle + sweep * t;
        let c = angle.cos();
        let s = angle.sin();
        out.push((center.0 + mx * c + vx * s, center.1 + my * c + vy * s));
    }
    out
}

fn polyline_with_bulges_points(
    points: &[(f64, f64)],
    bulges: &[f64],
    closed: bool,
    arc_segments: usize,
) -> Vec<(f64, f64)> {
    if points.len() <= 1 {
        return points.to_vec();
    }
    let mut bulge_values = vec![0.0f64; points.len()];
    for (idx, bulge) in bulges.iter().enumerate().take(points.len()) {
        bulge_values[idx] = *bulge;
    }

    let seg_count = if closed {
        points.len()
    } else {
        points.len().saturating_sub(1)
    };
    let mut out: Vec<(f64, f64)> = Vec::new();
    for idx in 0..seg_count {
        let start = points[idx];
        let end = points[(idx + 1) % points.len()];
        let bulge = bulge_values[idx];
        let segment = bulge_segment_points(start, end, bulge, arc_segments);
        append_segment_points(&mut out, &segment);
    }
    out
}

fn bulge_segment_points(
    start: (f64, f64),
    end: (f64, f64),
    bulge: f64,
    arc_segments: usize,
) -> Vec<(f64, f64)> {
    if bulge.abs() <= 1.0e-12 {
        return vec![start, end];
    }

    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let chord = (dx * dx + dy * dy).sqrt();
    if chord <= 1.0e-12 {
        return vec![start, end];
    }

    let theta = 4.0 * bulge.atan();
    if theta.abs() <= 1.0e-12 {
        return vec![start, end];
    }

    let normal = (-dy / chord, dx / chord);
    let center_offset = chord * (1.0 - bulge * bulge) / (4.0 * bulge);
    let mid = ((start.0 + end.0) * 0.5, (start.1 + end.1) * 0.5);
    let center = (
        mid.0 + normal.0 * center_offset,
        mid.1 + normal.1 * center_offset,
    );
    let radius = ((start.0 - center.0).powi(2) + (start.1 - center.1).powi(2)).sqrt();
    if radius <= 1.0e-12 {
        return vec![start, end];
    }

    let start_angle = (start.1 - center.1).atan2(start.0 - center.0);
    let segs = ((theta.abs() / std::f64::consts::TAU) * (arc_segments.max(8) as f64)).ceil();
    let segments = segs.max(2.0) as usize;
    let mut out = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let t = (i as f64) / (segments as f64);
        let angle = start_angle + theta * t;
        out.push((
            center.0 + radius * angle.cos(),
            center.1 + radius * angle.sin(),
        ));
    }
    if let Some(first) = out.first_mut() {
        *first = start;
    }
    if let Some(last) = out.last_mut() {
        *last = end;
    }
    out
}

fn normalized_sweep(start_angle: f64, end_angle: f64, is_ccw: bool) -> f64 {
    let mut sweep = end_angle - start_angle;
    if is_ccw {
        if sweep < 0.0 {
            sweep += std::f64::consts::TAU;
        }
    } else if sweep > 0.0 {
        sweep -= std::f64::consts::TAU;
    }
    sweep
}

fn points_equal_2d(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).abs() <= 1.0e-9 && (a.1 - b.1).abs() <= 1.0e-9
}

fn bounded_count(raw: u32, label: &str) -> Result<usize> {
    let count = raw as usize;
    if count > 1_000_000 {
        return Err(DwgError::new(
            ErrorKind::Format,
            format!("{} count is too large: {}", label, count),
        ));
    }
    Ok(count)
}
