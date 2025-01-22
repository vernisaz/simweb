use std::collections::HashMap;
use std::string::ToString;

#[derive(PartialEq, Debug, Copy, Clone, Default)]
enum TemplateState {
    #[default]
    InVal,
    VarStart, // $
    InVar,
}

pub trait Selectable {
    fn get_by_id(&self, id: &usize) -> String;
    fn get_by_name(&self, name: &String) -> String;
}

impl Selectable for Vec<Box<String>> {
    // was dyn Display
    fn get_by_id(&self, id: &usize) -> String {
        if let Some(el) = self.get(*id) {
            el.to_string()
        } else {
            format! {"${{{}}}", id}
        }
    }

    fn get_by_name(&self, name: &String) -> String {
        let id = name.parse::<usize>();
        if let Ok(id) = id {
            if let Some(el) = self.get(id) {
                el.to_string()
            } else {
                format! {"${{{}}}", id}
            }
        } else {
            format! {"${{{}}}", name}
        }
    }
}

impl Selectable for HashMap<String, String> {
    fn get_by_id(&self, id: &usize) -> String {
        let name = format! {"{}", id};
        if let Some(el) = self.get(&name) {
            el.to_string()
        } else {
            format! {"${{{}}}", id}
        }
    }

    fn get_by_name(&self, name: &String) -> String {
        if let Some(el) = self.get(name) {
            el.to_string()
        } else {
            format! {"${{{}}}", name}
        }
    }
}

impl Selectable for HashMap<String, Box<dyn ToString>> {
    // was Box<dyn Display> for String
    fn get_by_id(&self, id: &usize) -> String {
        let name = format! {"{}", id};
        if let Some(el) = self.get(&name) {
            el.to_string()
        } else {
            format! {"${{{}}}", id}
        }
    }

    fn get_by_name(&self, name: &String) -> String {
        if let Some(el) = self.get(name) {
            el.to_string()
        } else {
            format! {"${{{}}}", name}
        }
    }
}

pub fn interpolate(value: &String, args: &impl Selectable) -> String {
    let mut buf = Vec::with_capacity(4096);
    let mut buf_var = Vec::with_capacity(256); // buf for var name
    let chars = value.chars();
    let mut state = Default::default();
    for c in chars {
        match c {
            '$' => match state {
                TemplateState::InVal => state = TemplateState::VarStart,
                TemplateState::VarStart => buf.push(c),
                TemplateState::InVar => buf_var.push(c),
            },
            '{' => match state {
                TemplateState::VarStart => state = TemplateState::InVar,
                TemplateState::InVal => buf.push(c),
                TemplateState::InVar => buf_var.push(c),
            },
            '}' => match state {
                TemplateState::VarStart => {
                    state = TemplateState::InVal;
                    buf.push('$');
                    buf.push(c)
                }
                TemplateState::InVal => {
                    buf.push(c)
                }
                TemplateState::InVar => {
                    state = TemplateState::InVal;
                    let var: String = buf_var.clone().into_iter().collect();
                    let index = var.parse::<usize>();
                    let string = if let Ok(index) = index {
                        args.get_by_id(&index)
                    } else {
                        args.get_by_name(&var)
                    };
                    for vc in string.chars() {
                        buf.push(vc)
                    }
                    buf_var.clear()
                }
            },
            _ => match state {
                TemplateState::InVal => {
                    buf.push(c)
                }
                TemplateState::InVar => buf_var.push(c),
                TemplateState::VarStart => {
                    buf.push('$');
                    buf.push(c);
                    state = TemplateState::InVal
                }
            },
        }
    }
    buf.into_iter().collect()
}
