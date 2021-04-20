mod data;
mod template;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::ffi::OsStr;
use std::path::PathBuf;
use structopt::StructOpt;
use terminal_size::terminal_size;
use terminal_size::Width;

/// Fill out placeholders in templates
///
/// Reads a data file and a set of template-files and writes a set of output-files corresponding to
/// the templates files.
///
/// Placeholders in the template-files are replaced with values from the data file in the
/// output-files.
/// Each placeholder is marked up in the template-files by its name in double braces.
/// E.g. {{ recipient }}.
/// Whitespace surrounding the name is optional.
/// A single placeholder name may optionally recur multiple times in multiple in multiple template-files.
/// If so, they are replaced with the same value each time.
///
/// The data file is a headerless CSV file, read from STDIN.
/// Each record must have two fields - a placeholder-name and a value.
/// The delimiter is an ASCII comma.
/// Leading and trailing whitespace are trimmed from both placeholder-names and values.
/// Records are terminated with CR, LF or CRLF.
/// Fields may be quoted with ASCII double quote characters.
/// If you need to use an ASCII double quote you can escape it by doubling it.
/// If a record starts with a hash character, this line is ignored.
#[derive(Debug, StructOpt)]
struct Opt {
    /// A template-file - must have the .tmpl file extension.
    #[structopt(parse(from_os_str), name = "TEMPLATE-FILE")]
    template_files: Vec<PathBuf>,

    /// The directory where output-files are written - if none is given each output-file is
    /// written to the directory of its corresponding template-file
    #[structopt(short, long, parse(from_os_str), name = "DIR")]
    output_dir: Option<PathBuf>,
}

fn main() -> Result<()> {
    use std::collections::HashSet;
    use std::fs::read_to_string;
    use std::fs::OpenOptions;
    use std::io;
    use std::io::Read;
    use std::io::Write;
    use std::process::exit;

    let width = if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        120
    };

    let opt = Opt::from_clap(&Opt::clap().set_term_width(width).get_matches());

    let mut templates = vec![];
    let mut outputs = vec![];
    for template in &opt.template_files {
        if template.extension() == Some(OsStr::new("tmpl")) {
            let template = template
                .canonicalize()
                .with_context(|| format!("Failed to resolve template-file: {:?}", template))?;
            let output_dir = opt
                .output_dir
                .as_ref()
                .map(PathBuf::clone)
                .unwrap_or_else(|| template.parent().unwrap().to_path_buf());
            let output_file = template.file_stem().unwrap();
            outputs.push(output_dir.join(output_file));
            templates.push(template);
        } else {
            eprintln!(
                "Error: template-file must have .tmpl file extension: {:?}",
                template
            );
            exit(exitcode::USAGE);
        }
    }

    let templates: Vec<_> = templates
        .into_iter()
        .map(|path| {
            read_to_string(&path)
                .with_context(|| format!("Failed to read template-file {:?}", path))
                .map(move |corpus| (path, corpus))
        })
        .collect::<Result<_, _>>()?;

    let templates: Vec<_> = templates
        .iter()
        .map(|(path, corpus)| {
            template::parse(&corpus)
                .map_err(|(first, _)| first)
                .with_context(|| format!("Failed to parse template-file {:?}", path))
                .map(|tokens| (path, tokens))
        })
        .collect::<Result<_, _>>()?;

    let template_vars: HashSet<_> = templates
        .iter()
        .flat_map(|(_, tokens)| tokens.iter())
        .filter_map(|token| token.as_var())
        .map(|s| s.to_string())
        .collect();

    let mut input = Vec::new();
    io::stdin()
        .read_to_end(&mut input)
        .context("Failed to read data file from stdin")?;
    let input = input;
    let input_vars = data::parse(&input).context("Failed to validate data file")?;

    let extra = input_vars
        .keys()
        .filter(|name| !template_vars.contains(*name));
    for name in extra {
        eprintln!(
            "Warning: Unrecognized placeholder-name in input: {:?}",
            name
        );
    }
    let missing: Vec<_> = template_vars
        .iter()
        .filter(|name| !input_vars.contains_key(*name))
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(anyhow!(
            "Placeholders used in templates are not defined in data file: {:?}",
            missing
        )
        .context(format!(
            "Failed to validate data file against template-files {:?}",
            templates.iter().map(|(path, _)| path).collect::<Vec<_>>(),
        )));
    }

    let outputs: Vec<_> = outputs
        .into_iter()
        .map(|path| {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(false)
                .open(&path)
                .with_context(|| format!("Failed to open output file for writing {:?}", path))
                .map(move |handle| (path, handle))
        })
        .collect::<Result<_, _>>()?;

    for ((_, template), (path, mut output)) in templates.into_iter().zip(outputs) {
        output
            .set_len(0)
            .with_context(|| format!("Failed to truncate output file {:?}", path))?;
        for token in template {
            write!(output, "{}", token.eval(&input_vars))
                .with_context(|| format!("Failed writing to output file {:?}", path))?;
        }
    }

    Ok(())
}
