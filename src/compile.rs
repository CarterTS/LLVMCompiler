use std::io::Write;

use super::io::InputFile;
use super::cli::{Error, ErrorRecorder, Options};
use super::tokenizer::tokenize;
use super::irgen;
use super::codegen::{CodeGenerator, CodegenMode};

use super::parser::{ParseTreeNode, display_parse_tree};

/// Compile the given input file
pub fn compile(input: InputFile, options: &Options) -> Result<(), Error>
{
    let mut recorder: ErrorRecorder = ErrorRecorder::new();
    let data = input.data;
    let filename = input.filename;

    // Tokenization
    let tokens = tokenize(data, filename);

    // Parsing
    let node = recorder.wrap_return(super::parser::parse(tokens))?;

    if node.is_none()
    {
        Err(Error::fatal_error("No Parse Tree Returned"))?
    }

    if options.has_long_flag("tree") || options.has_short_flag("T")
    {
        println!("Parse Tree:");

        display_parse_tree(node.clone().unwrap(), String::new(), false);
    }

    // Determine Optimization Level
    let mut optimization_level = 0;

    if let Some(level) = options.map.get("-O")
    {
        if let Ok(val) = level[0].as_str().parse::<usize>()
        {
            optimization_level = val;
        }
        else
        {
            Err(Error::fatal_error(&format!("Bad optimization level '{}'", level[0])))?
        }
    }

    // Convert parse tree to IR
    let mut functions = vec![];

    match node.unwrap()
    {
        ParseTreeNode::Library(children) =>
        {
            for child in children
            {
                let mut function = irgen::Function::from_parse_tree_node(child)?;

                function = irgen::correct_types(function);

                function = irgen::optimize_function(function, optimization_level, !options.has_long_flag("nocomp"));

                functions.push(function);
            }
        },
        _ => {}
    }

    // Code Generation
    let mut codegen_mode = CodegenMode::IntermediateRepresentation;

    if let Some(name) = options.map.get("-g")
    {
        codegen_mode = CodegenMode::from_mode(&name[0]);
    }

    let output = CodeGenerator::new(codegen_mode, functions, options.clone()).render()?;

    // Display Output to stdout
    if options.has_long_flag("stdout")
    {
        println!("Output:\n{}", output);
    }

    // Output to a file
    else
    {
        let mut output_filename = "out.ll";

        if let Some(name) = options.map.get("-o")
        {
            output_filename = &name[0];
        }


        // Write to the output file
        let file = std::fs::File::create(output_filename);

        if file.is_err()
        {
            Err(Error::fatal_error(&format!("Could not create output file '{}'", output_filename)))?;
        }

        if let Err(_error) = write!(file.unwrap(), "{}", output)
        {
            Err(Error::fatal_error(&format!("Could not write to output file '{}'", output_filename)))?;
        }
    }
    
    Ok(())
}