pub fn contains(ty: &syn::Type, type_name: &syn::Ident) -> bool {
    match ty {
        syn::Type::Array(ty) => contains(&ty.elem, type_name),
        syn::Type::Group(ty) => contains(&ty.elem, type_name),
        syn::Type::Paren(ty) => contains(&ty.elem, type_name),
        syn::Type::Path(path) => match path.path.get_ident() {
            Some(ident) => *ident == *type_name,
            None => {
                for segment in &path.path.segments {
                    match &segment.arguments {
                        syn::PathArguments::AngleBracketed(generic_args) => {
                            for arg in &generic_args.args {
                                match &arg {
                                    syn::GenericArgument::Type(t) => {
                                        if contains(t, type_name) {
                                            return true;
                                        }
                                    }
                                    _ => continue,
                                }
                            }
                            continue;
                        }
                        _ => continue,
                    };
                }
                false
            }
        },
        syn::Type::Ptr(ty) => contains(&ty.elem, type_name),
        syn::Type::Reference(ty) => contains(&ty.elem, type_name),
        syn::Type::Slice(ty) => contains(&ty.elem, type_name),
        syn::Type::Tuple(ty) => ty.elems.iter().any(|elem| contains(elem, type_name)),
        _ => true,
    }
}
