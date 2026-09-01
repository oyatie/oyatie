#[derive(Clone, Debug, Eq, PartialEq)]
struct ReindeerProviderSourceEditV1 {
    start: usize,
    end: usize,
    replacement: String,
}

impl ReindeerProviderSourceEditV1 {
    fn replace(
        source: &str,
        span: proc_macro2::Span,
        replacement: String,
    ) -> Result<Self, ReindeerProviderAdaptationErrorV1> {
        let (start, end) = source_span_range_v1(source, span)?;
        Ok(Self {
            start,
            end,
            replacement,
        })
    }

    fn insert(
        source: &str,
        position: proc_macro2::LineColumn,
        replacement: String,
    ) -> Result<Self, ReindeerProviderAdaptationErrorV1> {
        let offset = source_offset_v1(source, position)?;
        Ok(Self {
            start: offset,
            end: offset,
            replacement,
        })
    }
}

fn source_span_range_v1(
    source: &str,
    span: proc_macro2::Span,
) -> Result<(usize, usize), ReindeerProviderAdaptationErrorV1> {
    let start = source_offset_v1(source, span.start())?;
    let end = source_offset_v1(source, span.end())?;
    if start > end {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    Ok((start, end))
}

fn source_offset_v1(
    source: &str,
    position: proc_macro2::LineColumn,
) -> Result<usize, ReindeerProviderAdaptationErrorV1> {
    if position.line == 0 {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    let line_start = if position.line == 1 {
        0
    } else {
        source
            .match_indices('\n')
            .nth(position.line - 2)
            .map(|(offset, _)| offset + 1)
            .ok_or(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)?
    };
    let line_bytes = source[line_start..]
        .find('\n')
        .unwrap_or(source.len() - line_start);
    if position.column > line_bytes {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    let offset = line_start
        .checked_add(position.column)
        .ok_or(ReindeerProviderAdaptationErrorV1::SourceTooLarge)?;
    if offset > source.len() || !source.is_char_boundary(offset) {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    Ok(offset)
}

fn apply_source_edits_v1(
    source: &str,
    mut edits: Vec<ReindeerProviderSourceEditV1>,
) -> Result<Vec<u8>, ReindeerProviderAdaptationErrorV1> {
    edits.sort_by_key(|edit| (edit.start, edit.end));
    if edits.iter().any(|edit| {
        edit.start > edit.end
            || edit.end > source.len()
            || !source.is_char_boundary(edit.start)
            || !source.is_char_boundary(edit.end)
    }) {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    if edits
        .windows(2)
        .any(|pair| pair[0].end > pair[1].start || pair[0].start == pair[1].start)
    {
        return Err(ReindeerProviderAdaptationErrorV1::OverlappingSourceEdit);
    }
    let growth = edits
        .iter()
        .map(|edit| edit.replacement.len().saturating_sub(edit.end - edit.start))
        .try_fold(0usize, usize::checked_add)
        .ok_or(ReindeerProviderAdaptationErrorV1::OutputTooLarge)?;
    let capacity = source
        .len()
        .checked_add(growth)
        .ok_or(ReindeerProviderAdaptationErrorV1::OutputTooLarge)?;
    if capacity > MAX_PROVIDER_OUTPUT_BYTES_V1 {
        return Err(ReindeerProviderAdaptationErrorV1::OutputTooLarge);
    }

    let mut output = source.to_owned();
    for edit in edits.into_iter().rev() {
        output.replace_range(edit.start..edit.end, &edit.replacement);
    }
    syn::parse_file(&output)
        .map_err(|_| ReindeerProviderAdaptationErrorV1::GeneratedSourceInvalid)?;
    Ok(output.into_bytes())
}

fn exact_source_fragment_edits_v1(
    source: &str,
    preimage: &str,
    postimage: &str,
    expected_matches: usize,
) -> Result<Vec<ReindeerProviderSourceEditV1>, ReindeerProviderAdaptationErrorV1> {
    if preimage.is_empty() || expected_matches == 0 {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    let matches = source
        .match_indices(preimage)
        .map(|(start, _)| ReindeerProviderSourceEditV1 {
            start,
            end: start + preimage.len(),
            replacement: postimage.to_owned(),
        })
        .collect::<Vec<_>>();
    if matches.len() != expected_matches {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    Ok(matches)
}

fn render_provider_module_v1(
    tokens: proc_macro2::TokenStream,
) -> Result<Vec<u8>, ReindeerProviderAdaptationErrorV1> {
    let syntax = syn::parse2::<syn::File>(tokens)
        .map_err(|_| ReindeerProviderAdaptationErrorV1::GeneratedSourceInvalid)?;
    let output = prettyplease::unparse(&syntax).into_bytes();
    if output.len() > MAX_PROVIDER_OUTPUT_BYTES_V1 {
        return Err(ReindeerProviderAdaptationErrorV1::OutputTooLarge);
    }
    Ok(output)
}

#[cfg(test)]
mod provider_source_edit_tests_v1 {
    use super::{
        ReindeerProviderAdaptationErrorV1, ReindeerProviderSourceEditV1, apply_source_edits_v1,
        exact_source_fragment_edits_v1,
    };

    #[test]
    fn overlapping_semantic_edits_refuse_before_mutation() {
        let source = "const VALUE: u8 = 1;\n";
        let edits = vec![
            ReindeerProviderSourceEditV1 {
                start: 6,
                end: 11,
                replacement: "FIRST".to_owned(),
            },
            ReindeerProviderSourceEditV1 {
                start: 8,
                end: 13,
                replacement: "SECOND".to_owned(),
            },
        ];

        assert_eq!(
            apply_source_edits_v1(source, edits),
            Err(ReindeerProviderAdaptationErrorV1::OverlappingSourceEdit)
        );
    }

    #[test]
    fn exact_fragment_edits_require_the_declared_cardinality() {
        let source = "const FIRST: &str = \"old\";\nconst SECOND: &str = \"old\";\n";
        let edits = exact_source_fragment_edits_v1(source, "\"old\"", "\"changed\"", 2).unwrap();
        assert_eq!(
            apply_source_edits_v1(source, edits).unwrap(),
            b"const FIRST: &str = \"changed\";\nconst SECOND: &str = \"changed\";\n"
        );
        assert_eq!(
            exact_source_fragment_edits_v1(source, "\"old\"", "\"changed\"", 1),
            Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)
        );
    }
}
