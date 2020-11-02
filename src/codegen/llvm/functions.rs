use crate::cli::Error;

use crate::irgen::{Function, DataType, NonPtrType, Symbol, Value, OpCode, get_value_type};

use super::{convert_to_llvm, bytes_size_of};

use std::collections::HashMap;

/// A value held in llvm
pub struct LLVMValue
{
    pub ptr: String,
    pub datatype: DataType
}

impl LLVMValue
{
    /// Generate a new LLVM Value object
    pub fn new(ptr: String, datatype: DataType) -> Self
    {
        Self
        {
            ptr,
            datatype
        }
    }

    /// Retrieve the datatype of the object
    pub fn get_datatype(&self) -> DataType
    {
        self.datatype
    }

    /// Retrieve the datatype of the pointer
    pub fn get_pointer_datatype(&self) -> DataType
    {
        let mut datatype = self.get_datatype().clone();
        datatype.num_ptr += 1;

        datatype
    }
}

/// A wrapper for giving a context to code generation for an LLVM function
pub struct FunctionGenerationContext
{
    func: Function,
    values: HashMap<String, LLVMValue>,
    next_temp: usize,
    result: String
}

impl FunctionGenerationContext
{
    /// Generate a new function generation context object
    pub fn new(func: Function) -> Self
    {
        Self
        {
            func,
            values: HashMap::new(),
            next_temp: 0,
            result: String::new()
        }
    }

    /// Insert a new command
    pub fn insert_command(&mut self, cmd: &str)
    {
        self.result += &format!("    {}\n", cmd);
    }

    /// Insert a label
    pub fn insert_label(&mut self, label: &str)
    {
        let l = self.render_value(&Value::Label(String::from(label)));
        self.insert_command(&format!("br {}", l));
        
        self.result += &format!("\n  {}:\n", label);
    }

    /// Get the next temporary variable
    pub fn get_next_temp(&mut self) -> String
    {
        self.next_temp += 1;
        format!("%V{}", self.next_temp - 1)
    }

    /// Create a new value
    pub fn create_new_value(&mut self, title: String, datatype: DataType)
    {
        // Create the raw object
        let ptr = self.get_next_temp();
        let new_value = LLVMValue::new(ptr, datatype);
        self.values.insert(title.clone(), new_value);

        let dt = self.values.get(&title).unwrap().get_datatype();
        let ptr = self.values.get(&title).unwrap().ptr.clone();

        // Add the command
        self.insert_command(
            &format!("{} = alloca {}, align {}", 
                            ptr, 
                            convert_to_llvm(&dt), 
                            bytes_size_of(&dt)));
    }

    /// Get the reference for a variable
    pub fn get_reference(&mut self, var: &Symbol, include_type: bool) -> String
    {
        // If the variable is not already stored, create it
        if !self.values.contains_key(&var.title)
        {
            self.create_new_value(var.title.clone(), var.datatype);
        }
        let ptr = self.values.get(&var.title).unwrap().ptr.clone();
        let pdt = self.values.get(&var.title).unwrap().get_pointer_datatype();

        if include_type
        {
            format!("{} {}",convert_to_llvm(&pdt), ptr)
        }
        else
        {
            ptr
        }
    }

    /// Get the value for a variable
    pub fn get_value(&mut self, var: &Symbol, include_type: bool) -> String
    {
        // If the variable is not already stored, create it
        if !self.values.contains_key(&var.title)
        {
            self.create_new_value(var.title.clone(), var.datatype);
        }
        
        let reg = self.get_next_temp();
        let dt = self.values.get(&var.title).unwrap().get_datatype();
        let pdt = self.values.get(&var.title).unwrap().get_pointer_datatype();
        let ptr = self.values.get(&var.title).unwrap().ptr.clone();

        self.insert_command(&format!("{} = load {}, {} {}, align {}", 
                                           reg, 
                                           convert_to_llvm(&dt),
                                           convert_to_llvm(&pdt),
                                           ptr,
                                           bytes_size_of(&var.datatype)));

        if include_type
        {
            if !(dt.raw_type == NonPtrType::Void && dt.num_ptr == 0)
            {
                format!("{} {}", convert_to_llvm(&dt), reg)
            }
            else
            {
                format!("{}", convert_to_llvm(&dt))
            }
        }
        else
        {
            reg
        }
    }

    /// Render a value for direct insertion into a command
    pub fn render_value(&mut self, val: &Value) -> String
    {
        match val
        {
            Value::Label(label) =>
            {
                format!("label %{}", label)
            },
            Value::Literal(literal) =>
            {
                // If the type isn't void
                if !(literal.datatype.raw_type == NonPtrType::Void && literal.datatype.num_ptr == 0)
                {
                    if literal.datatype.num_ptr == 0 && !literal.datatype.is_ref
                    {
                        format!("{} {}", convert_to_llvm(&literal.datatype), literal.value)
                    }
                    else
                    {
                        format!("{0} inttoptr (i64 {1} to {0})", convert_to_llvm(&literal.datatype), literal.value)
                    }
                }
                else
                {
                    format!("{}", convert_to_llvm(&literal.datatype))
                }
            },
            Value::Symbol(symbol) =>
            {
                self.get_value(symbol, true)
            }
        }
    }

    /// Render a pointer to a value for direct insertion into a command
    pub fn render_pointer(&mut self, val: &Value) -> String
    {
        match val
        {
            Value::Label(_) =>
            {
                panic!("The pointer of a label?!")
            },
            Value::Literal(_) =>
            {
                panic!("The pointer of a literal?!")
            },
            Value::Symbol(symbol) =>
            {
                self.get_reference(symbol, true)
            }
        }
    }

    /// Add a move via the syntax of the 'store' command
    pub fn add_move(&mut self, dest: &Value, src: String)
    {
        if let Some(datatype) = get_value_type(dest)
        {
            // If the data type is not a reference, just store the value into a pointer to the first
            if !datatype.is_ref
            {
                let val0 = self.render_pointer(dest);
                self.insert_command(
                            &format!("store {}, {}", 
                                        src,
                                        val0));
            }
            // Otherwise, the target *is* the pointer
            else
            {
                let val0 = self.render_value(dest);
                self.insert_command(
                            &format!("store {}, {}", 
                                        src,
                                        val0));
            }
        }
    }

    /// Render an IR function in LLVM IR
    pub fn render_function(&mut self) -> Result<String, Error>
    {
        // Clone the function to avoid borrow issues later
        let func = self.func.clone();

        self.result = String::new();

        // Function return type and name
        self.result += &format!("define {} @{}", convert_to_llvm(&func.return_type), func.name);

        // Arguments
        self.result += "(";

        for (i, (name, datatype)) in func.arguments.iter().enumerate()
        {
            self.result += &format!("{} %{}", convert_to_llvm(datatype), name);

            if i < func.arguments.len() - 1
            {
                self.result += ", ";
            }
        }

        self.result += ")\n";

        // Body

        self.result += "{\n";

        // Allocate all of the space required for the symbols
        for symbol in func.get_all_symbols()
        {
            self.create_new_value(symbol.title, symbol.datatype);
        }

        // Go over every instruction
        for i in 0..self.func.instructions.len()
        {
            let labels = if let Some(l) = func.labels.get(&i).clone() {l.clone()} else {vec![]};
            
            for label in labels
            {
                self.insert_label(label.as_str());
            }

            if let Some(inst) = &func.instructions.get(&i)
            {
                match inst.opcode
                {
                    // Return Command
                    OpCode::Ret =>
                    {
                        let val = self.render_value(&inst.arguments[0]).clone();
                        self.insert_command(&format!("ret {}", val));
                    },
                    // Move or Allocate
                    OpCode::Mov | OpCode::Alloc =>
                    {
                        let val = self.render_value(&inst.arguments[1]);
                        self.add_move(&inst.arguments[0], val);
                    },
                    // Dereference Command
                    OpCode::Deref =>
                    {
                        if let Value::Symbol(var) = &inst.arguments[0]
                        {
                            let reg = self.get_next_temp();
                            let dt = self.values.get(&var.title).unwrap().get_datatype();

                            let val = self.render_value(&inst.arguments[1]);

                            self.insert_command(&format!("{} = load {}, {}, align {}", 
                                            reg, 
                                            convert_to_llvm(&dt),
                                            val,
                                            bytes_size_of(&var.datatype)));

                            self.add_move(&inst.arguments[0], format!("{} {}", convert_to_llvm(&dt), reg));
                        };
                    },
                    // Unconditional Jump
                    OpCode::Jmp =>
                    {
                        let label = self.render_value(&inst.arguments[0]);
                        self.insert_command(&format!("br {}", label));
                    },
                    _ => {println!("Not handling instruction {}", inst);}
                }
            }
        }

        self.result += "}\n";

        Ok(self.result.clone())
    }
}