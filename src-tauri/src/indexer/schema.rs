use tantivy::schema::*;

pub struct IndexSchema {
    pub schema: Schema,
    pub f_path: Field,
    pub f_name: Field,
    pub f_content: Field,
    pub f_ext: Field,
    pub f_size: Field,
    pub f_modified: Field,
    pub f_indexed: Field,
}

/// TextOptions para o campo de conteúdo: tokenizer "default", posições, não STORED.
pub fn get_text_tokenizer_options() -> TextOptions {
    TextOptions::default().set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("default")
            .set_index_option(IndexRecordOption::WithFreqsAndPositions),
    )
}

pub fn build_schema() -> IndexSchema {
    let mut b = Schema::builder();

    // path: tokenizer "raw" (match exato) + STORED
    let path_opts = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("raw")
                .set_index_option(IndexRecordOption::Basic),
        )
        .set_stored();
    let f_path = b.add_text_field("path", path_opts);

    // name: tokenizado (default) + STORED
    let f_name = b.add_text_field("name", TEXT | STORED);

    // content: tokenizado, não STORED
    let f_content = b.add_text_field("content", get_text_tokenizer_options());

    // ext: string (raw) + STORED + FAST
    let f_ext = b.add_text_field("ext", STRING | STORED | FAST);

    // numéricos: STORED + FAST
    let num_opts: NumericOptions = NumericOptions::default().set_stored().set_fast();
    let f_size = b.add_u64_field("size", num_opts.clone());
    let f_modified = b.add_u64_field("modified", num_opts.clone());
    let f_indexed = b.add_u64_field("indexed", num_opts);

    let schema = b.build();

    IndexSchema {
        schema,
        f_path,
        f_name,
        f_content,
        f_ext,
        f_size,
        f_modified,
        f_indexed,
    }
}
