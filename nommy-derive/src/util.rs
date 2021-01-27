pub fn type_contains_generic(ty: &syn::Type, args: &Vec<syn::Ident>) -> bool {
    match ty {
        syn::Type::Array(ty) => type_contains_generic(&ty.elem, args),
        syn::Type::Group(ty) => type_contains_generic(&ty.elem, args),
        syn::Type::Paren(ty) => type_contains_generic(&ty.elem, args),
        syn::Type::Path(path) => match path.path.get_ident() {
            Some(ident) => args.contains(ident),
            None => {
                for segment in &path.path.segments {
                    match &segment.arguments {
                        syn::PathArguments::AngleBracketed(generic_args) => {
                            for arg in &generic_args.args {
                                match &arg {
                                    syn::GenericArgument::Type(t) => {
                                        if type_contains_generic(t, args) {
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
        syn::Type::Ptr(ty) => type_contains_generic(&ty.elem, args),
        syn::Type::Reference(ty) => type_contains_generic(&ty.elem, args),
        syn::Type::Slice(ty) => type_contains_generic(&ty.elem, args),
        syn::Type::Tuple(ty) => ty
            .elems
            .iter()
            .any(|elem| type_contains_generic(elem, args)),
        _ => true,
    }
}
