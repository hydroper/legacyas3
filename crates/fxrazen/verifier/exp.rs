use crate::ns::*;

pub(crate) struct ExpSubverifier;

impl ExpSubverifier {
    // QualifiedIdentifier - returns (ns, local name)
    pub fn verify_qualified_identifier(verifier: &mut Subverifier, id: &QualifiedIdentifier) -> Result<Option<(Option<Thingy>, PropertyLookupKey)>, DeferError> {
        let QualifiedIdentifier { qualifier, id, .. } = id;

        let mut failed = false;

        let mut result_qual: Option<Thingy> = None;

        if let Some(qualifier) = qualifier {
            result_qual = verifier.imp_coerce_exp(qualifier, &verifier.host.namespace_type())?;
            if result_qual.is_none() {
                failed = true;
            }
        }

        let mut result_key: Option<PropertyLookupKey> = None;

        match id {
            QualifiedIdentifierIdentifier::Id((id, _)) => {
                result_key = Some(PropertyLookupKey::LocalName(id.clone()));
            },
            QualifiedIdentifierIdentifier::Brackets(exp) => {
                let v = verifier.imp_coerce_exp(exp, &verifier.host.string_type())?;
                if let Some(v) = v {
                    result_key = Some(PropertyLookupKey::Computed(v));
                } else {
                    failed = true;
                }
            },
        }

        if failed {
            Ok(None)
        } else {
            Ok(Some((result_qual, result_key.unwrap())))
        }
    }

    // QualifiedIdentifier
    pub fn verify_qualified_identifier_as_exp(verifier: &mut Subverifier, id: &QualifiedIdentifier, context: &VerifierExpressionContext) -> Result<Option<Thingy>, DeferError> {
        // Check for inline constants
        if let Some((name, cdata)) = Self::filter_inline_constant(verifier, id) {
            // Defer
            verifier.host.string_type().defer()?;
            verifier.host.non_null_primitive_types()?;

            return Ok(Self::verify_inline_constant(verifier, &id.location, name, cdata, context));
        }

        let qn = Self::verify_qualified_identifier(verifier, id)?;
        if qn.is_none() {
            return Ok(None);
        }
        let (qual, key) = qn.unwrap();

        // Attribute
        if id.attribute {
            return Ok(Some(verifier.host.factory().create_dynamic_scope_reference_value(&verifier.scope(), qual, &key.computed_or_local_name(&verifier.host)?)));
        }

        let r = verifier.scope().lookup_in_scope_chain(&verifier.host, qual, &key);
        if r.is_err() {
            match r.unwrap_err() {
                PropertyLookupError::AmbiguousReference(name) => {
                    verifier.add_verify_error(&id.location, FxDiagnosticKind::AmbiguousReference, diagarg![name.clone()]);
                    return Ok(None);
                },
                PropertyLookupError::Defer => {
                    return Err(DeferError());
                },
                PropertyLookupError::VoidBase => {
                    verifier.add_verify_error(&id.location, FxDiagnosticKind::AccessOfVoid, diagarg![]);
                    return Ok(None);
                },
                PropertyLookupError::NullableObject { .. } => {
                    verifier.add_verify_error(&id.location, FxDiagnosticKind::AccessOfNullable, diagarg![]);
                    return Ok(None);
                },
            }
        }
        let r = r.unwrap();
        if r.is_none() {
            verifier.add_verify_error(&id.location, FxDiagnosticKind::UndefinedProperty, diagarg![key.local_name().unwrap()]);
            return Ok(None);
        }
        let r = r.unwrap();

        // Mark local capture
        verifier.detect_local_capture(&r);

        // Post-processing
        verifier.reference_post_processing(r, context)
    }

    fn filter_inline_constant(verifier: &mut Subverifier, id: &QualifiedIdentifier) -> Option<(String, String)> {
        let QualifiedIdentifier { qualifier, id, .. } = id;

        if let Some(qualifier) = qualifier {
            // Detect any inline constant
            let inlinekqid = qualifier.to_identifier_name().map(|name| name.0);
            let inlinekln = if let QualifiedIdentifierIdentifier::Id((name, _)) = id { Some(name) } else { None };
            if let (Some(inlinekqid), Some(inlinekln)) = (inlinekqid, inlinekln) {
                let inlinekid = format!("{}::{}", inlinekqid, inlinekln);
                if let Some(cdata) = verifier.host.config_constants().get(&inlinekid) {
                    return Some((inlinekid, cdata));
                }
            }
        }

        None
    }

    fn verify_inline_constant(verifier: &mut Subverifier, location: &Location, name: String, mut cdata: String, context: &VerifierExpressionContext) -> Option<Thingy> {
        cdata = cdata.trim().to_owned();

        if ["true", "false"].contains(&cdata.as_str()) {
            let boolean_type = verifier.host.boolean_type();
            if boolean_type.is::<UnresolvedThingy>() {
                verifier.add_verify_error(location, FxDiagnosticKind::CouldNotExpandInlineConstant, diagarg![]);
                return None;
            }
            return Some(verifier.host.factory().create_boolean_constant(cdata == "true", &boolean_type));
        }

        // Cache compilation unit for less memory usage
        let cu: Rc<CompilationUnit>;
        if let Some(cu1) = verifier.host.config_constants_cu().get(&name) {
            cu = cu1;
        } else {
            cu = CompilationUnit::new(None, cdata);
            verifier.host.config_constants_cu().set(name, cu.clone());
        }

        // An expression is always built for the inline constant,
        // which must be a compile-time constant.
        let exp = ParserFacade(&cu, ParserOptions::default()).parse_expression();
        if cu.invalidated() {
            verifier.add_verify_error(location, FxDiagnosticKind::CouldNotExpandInlineConstant, diagarg![]);
            return None;
        }
        let Ok(cval) = verifier.verify_expression(&exp, context) else {
            verifier.add_verify_error(location, FxDiagnosticKind::CouldNotExpandInlineConstant, diagarg![]);
            return None;
        };
        if let Some(cval) = cval.as_ref() {
            if !cval.is::<Constant>() {
                verifier.add_verify_error(location, FxDiagnosticKind::CouldNotExpandInlineConstant, diagarg![]);
                return None;
            }
        }
        cval
    }

    pub fn verify_null_literal(verifier: &mut Subverifier, literal: &NullLiteral, context: &VerifierExpressionContext) -> Result<Option<Thingy>, DeferError> {
        if let Some(t) = context.context_type.as_ref() {
            if t.includes_null(&verifier.host)? {
                return Ok(Some(verifier.host.factory().create_null_constant(t)));
            } else {
                verifier.add_verify_error(&literal.location, FxDiagnosticKind::NullNotExpectedHere, diagarg![]);
                return Ok(None);
            }
        }
        Ok(Some(verifier.host.factory().create_null_constant(&verifier.host.any_type())))
    }

    pub fn verify_boolean_literal(verifier: &mut Subverifier, literal: &BooleanLiteral, context: &VerifierExpressionContext) -> Result<Option<Thingy>, DeferError> {
        if let Some(t) = context.context_type.as_ref() {
            let t_esc = t.escape_of_nullable_or_non_nullable();
            if [verifier.host.any_type(), verifier.host.object_type().defer()?, verifier.host.boolean_type().defer()?].contains(&t_esc) {
                return Ok(Some(verifier.host.factory().create_boolean_constant(literal.value, &t)));
            }
        }
        Ok(Some(verifier.host.factory().create_boolean_constant(literal.value, &verifier.host.boolean_type().defer()?)))
    }

    pub fn verify_numeric_literal(verifier: &mut Subverifier, literal: &NumericLiteral, context: &VerifierExpressionContext) -> Result<Option<Thingy>, DeferError> {
        if let Some(t) = context.context_type.as_ref() {
            let t_esc = t.escape_of_nullable_or_non_nullable();
            if verifier.host.numeric_types()?.contains(&t_esc) {
                let n = Self::parse_number_as_data_type(&verifier.host, literal, &t_esc, context);
                if n.is_err() {
                    verifier.add_verify_error(&literal.location, FxDiagnosticKind::CouldNotParseNumber, diagarg![t_esc]);
                    return Ok(None);
                }
                return Ok(Some(verifier.host.factory().create_number_constant(n.unwrap(), t)));
            }
        }
        let t = verifier.host.number_type().defer()?;
        let n = Self::parse_number_as_data_type(&verifier.host, literal, &t, context);
        if n.is_err() {
            verifier.add_verify_error(&literal.location, FxDiagnosticKind::CouldNotParseNumber, diagarg![t]);
            return Ok(None);
        }
        return Ok(Some(verifier.host.factory().create_number_constant(n.unwrap(), &t)));
    }

    pub fn parse_number_as_data_type(host: &SemanticHost, literal: &NumericLiteral, data_type: &Thingy, context: &VerifierExpressionContext) -> Result<NumberVariant, ParserError> {
        if data_type == &host.number_type() {
            Ok(NumberVariant::Number(literal.parse_double(context.preceded_by_negative)?))
        } else if data_type == &host.float_type() {
            Ok(NumberVariant::Float(literal.parse_float(context.preceded_by_negative)?))
        } else if data_type == &host.int_type() {
            Ok(NumberVariant::Int(literal.parse_int(context.preceded_by_negative)?))
        } else {
            assert!(data_type == &host.uint_type());
            Ok(NumberVariant::Uint(literal.parse_uint()?))
        }
    }

    pub fn verify_string_literal(verifier: &mut Subverifier, literal: &StringLiteral, context: &VerifierExpressionContext) -> Result<Option<Thingy>, DeferError> {
        if let Some(t) = context.context_type.as_ref() {
            let t_esc = t.escape_of_nullable_or_non_nullable();
            if t_esc.is::<EnumType>() {
                let slot = t_esc.enum_member_slot_mapping().get(&literal.value);
                if let Some(slot) = slot {
                    let k = verifier.host.factory().create_static_reference_value(&t_esc, &slot)?;
                    return Ok(TypeConversions(&verifier.host).implicit(&k, &t, false)?);
                } else {
                    verifier.add_verify_error(&literal.location, FxDiagnosticKind::NoMatchingEnumMember, diagarg![literal.value.clone(), t_esc]);
                    return Ok(None);
                }
            }
        }
        return Ok(Some(verifier.host.factory().create_string_constant(literal.value.clone(), &verifier.host.string_type().defer()?)));
    }

    pub fn verify_this_literal(verifier: &mut Subverifier, literal: &ThisLiteral) -> Result<Option<Thingy>, DeferError> {
        let activation = verifier.scope().search_activation();
        if activation.is_some() && activation.as_ref().unwrap().this().is_some() {
            Ok(activation.clone().unwrap().this())
        } else {
            verifier.add_verify_error(&literal.location, FxDiagnosticKind::UnexpectedThis, diagarg![]);
            Ok(None)
        }
    }

    pub fn verify_reg_exp_literal(verifier: &mut Subverifier, _literal: &RegExpLiteral, context: &VerifierExpressionContext) -> Result<Option<Thingy>, DeferError> {
        if let Some(t) = context.context_type.as_ref() {
            let t_esc = t.escape_of_nullable_or_non_nullable();
            if [verifier.host.any_type(), verifier.host.object_type().defer()?, verifier.host.reg_exp_type().defer()?].contains(&t_esc) {
                return Ok(Some(verifier.host.factory().create_value(&t)));
            }
        }
        Ok(Some(verifier.host.factory().create_value(&verifier.host.reg_exp_type().defer()?)))
    }

    pub fn verify_xml_exp(verifier: &mut Subverifier, exp: &XmlExpression, context: &VerifierExpressionContext) -> Result<Option<Thingy>, DeferError> {
        Self::verify_xml_elem(verifier, &exp.element)?;
        if let Some(t) = context.context_type.as_ref() {
            let t_esc = t.escape_of_nullable_or_non_nullable();
            if [verifier.host.any_type(), verifier.host.object_type().defer()?, verifier.host.xml_type().defer()?].contains(&t_esc) {
                return Ok(Some(verifier.host.factory().create_value(&t)));
            }
        }
        Ok(Some(verifier.host.factory().create_value(&verifier.host.xml_type().defer()?)))
    }

    pub fn verify_xml_list_exp(verifier: &mut Subverifier, exp: &XmlListExpression, context: &VerifierExpressionContext) -> Result<Option<Thingy>, DeferError> {
        for content in exp.content.iter() {
            Self::verify_xml_content(verifier, content)?;
        }
        if let Some(t) = context.context_type.as_ref() {
            let t_esc = t.escape_of_nullable_or_non_nullable();
            if [verifier.host.any_type(), verifier.host.object_type().defer()?, verifier.host.xml_list_type().defer()?].contains(&t_esc) {
                return Ok(Some(verifier.host.factory().create_value(&t)));
            }
        }
        Ok(Some(verifier.host.factory().create_value(&verifier.host.xml_list_type().defer()?)))
    }

    pub fn verify_xml_elem(verifier: &mut Subverifier, elem: &XmlElement) -> Result<(), DeferError> {
        if let XmlTagName::Expression(exp) = &elem.name {
            verifier.verify_expression(exp, &VerifierExpressionContext { ..default() })?;
        }
        for attr in &elem.attributes {
            if let XmlAttributeValue::Expression(exp) = &attr.value {
                verifier.verify_expression(exp, &VerifierExpressionContext { ..default() })?;
            }
        }
        if let Some(exp) = &elem.attribute_expression {
            verifier.verify_expression(exp, &VerifierExpressionContext { ..default() })?;
        }
        if let Some(content_list) = &elem.content {
            for content in content_list {
                Self::verify_xml_content(verifier, content)?;
            }
        }
        if let Some(XmlTagName::Expression(exp)) = &elem.closing_name {
            verifier.verify_expression(exp, &VerifierExpressionContext { ..default() })?;
        }
        Ok(())
    }

    pub fn verify_xml_content(verifier: &mut Subverifier, content: &Rc<XmlContent>) -> Result<(), DeferError> {
        match content.as_ref() {
            XmlContent::Element(elem) => {
                Self::verify_xml_elem(verifier, elem)?;
                Ok(())
            },
            XmlContent::Expression(exp) => {
                verifier.verify_expression(exp, &VerifierExpressionContext { ..default() })?;
                Ok(())
            },
            _ => Ok(()),
        }
    }

    pub fn verify_new_exp(verifier: &mut Subverifier, exp: &NewExpression) -> Result<Option<Thingy>, DeferError> {
        let Some(base) = verifier.verify_expression(&exp.base, &default())? else {
            if let Some(arguments) = &exp.arguments {
                for arg in arguments.iter() {
                    verifier.verify_expression(arg, &default())?;
                }
            }
            return Ok(None);
        };

        if let Some(t) = base.as_type() {
            if !(t.is_class_type_possibly_after_sub() && !t.is_static() && !t.is_abstract()) {
                verifier.add_verify_error(&exp.base.location(), FxDiagnosticKind::UnexpectedNewBase, diagarg![]);

                if let Some(arguments) = &exp.arguments {
                    for arg in arguments.iter() {
                        verifier.verify_expression(arg, &default())?;
                    }
                }

                return Ok(Some(verifier.host.factory().create_value(&verifier.host.any_type())));
            }

            // In AS3, the constructor is not inherited unlike in other languages.
            let ctor = t.constructor_method(&verifier.host);

            if let Some(ctor) = ctor {
                let sig = ctor.signature(&verifier.host).defer()?;
                match ArgumentsSubverifier::verify(verifier, exp.arguments.as_ref().unwrap_or(&vec![]), &sig) {
                    Ok(_) => {},
                    Err(VerifierArgumentsError::Defer) => {
                        return Err(DeferError());
                    },
                    Err(VerifierArgumentsError::Expected(n)) => {
                        verifier.add_verify_error(&exp.base.location(), FxDiagnosticKind::IncorrectNumArguments, diagarg![n.to_string()]);
                    },
                    Err(VerifierArgumentsError::ExpectedNoMoreThan(n)) => {
                        verifier.add_verify_error(&exp.base.location(), FxDiagnosticKind::IncorrectNumArgumentsNoMoreThan, diagarg![n.to_string()]);
                    },
                }
            } else {
                if let Some(arguments) = &exp.arguments {
                    if !arguments.is_empty() {
                        verifier.add_verify_error(&exp.base.location(), FxDiagnosticKind::IncorrectNumArgumentsNoMoreThan, diagarg!["0".to_string()]);
                    }
                    for arg in arguments.iter() {
                        verifier.verify_expression(arg, &default())?;
                    }
                }
            }

            return Ok(Some(verifier.host.factory().create_value(&t)));
        }

        let base_st = base.static_type(&verifier.host);
        let base_st_esc = base_st.escape_of_non_nullable();

        if ![verifier.host.any_type(), verifier.host.class_type().defer()?].contains(&base_st_esc) {
            verifier.add_verify_error(&exp.base.location(), FxDiagnosticKind::UnexpectedNewBase, diagarg![]);
        }

        if let Some(arguments) = &exp.arguments {
            for arg in arguments.iter() {
                verifier.verify_expression(arg, &default())?;
            }
        }

        return Ok(Some(verifier.host.factory().create_value(&verifier.host.any_type())));
    }

    pub fn verify_member_exp(verifier: &mut Subverifier, exp: &Rc<Expression>, member_exp: &MemberExpression, context: &VerifierExpressionContext) -> Result<Option<Thingy>, DeferError> {
        // Shadowing package names
        if let Some(r) = Self::verify_member_exp_pre_package_names(verifier, exp, member_exp)? {
            return Ok(Some(r));
        }

        let id = &member_exp.identifier;

        let Some(base) = verifier.verify_expression(&member_exp.base, &default())? else {
            Self::verify_qualified_identifier(verifier, id)?;
            return Ok(None);
        };

        let qn = Self::verify_qualified_identifier(verifier, id)?;
        if qn.is_none() {
            return Ok(None);
        }
        let (qual, key) = qn.unwrap();

        // Attribute
        if id.attribute {
            return Ok(Some(verifier.host.factory().create_dynamic_reference_value(&base, qual, &key.computed_or_local_name(&verifier.host)?)));
        }

        let open_ns_set = verifier.scope().concat_open_ns_set_of_scope_chain();
        let r = PropertyLookup(&verifier.host).lookup_in_object(&base, &open_ns_set, qual, &key);
        if r.is_err() {
            match r.unwrap_err() {
                PropertyLookupError::AmbiguousReference(name) => {
                    verifier.add_verify_error(&id.location, FxDiagnosticKind::AmbiguousReference, diagarg![name.clone()]);
                    return Ok(None);
                },
                PropertyLookupError::Defer => {
                    return Err(DeferError());
                },
                PropertyLookupError::VoidBase => {
                    verifier.add_verify_error(&id.location, FxDiagnosticKind::AccessOfVoid, diagarg![]);
                    return Ok(None);
                },
                PropertyLookupError::NullableObject { .. } => {
                    verifier.add_verify_error(&id.location, FxDiagnosticKind::AccessOfNullable, diagarg![]);
                    return Ok(None);
                },
            }
        }
        let r = r.unwrap();
        if r.is_none() {
            verifier.add_verify_error(&id.location, FxDiagnosticKind::UndefinedPropertyWithStaticType, diagarg![key.local_name().unwrap(), base.static_type(&verifier.host)]);
            return Ok(None);
        }
        let r = r.unwrap();

        // No need to mark local capture for the property operator.
        // verifier.detect_local_capture(&r);

        // Post-processing
        verifier.reference_post_processing(r, context)
    }

    fn verify_member_exp_pre_package_names(verifier: &mut Subverifier, exp: &Rc<Expression>, member_exp: &MemberExpression) -> Result<Option<Thingy>, DeferError> {
        let Some(dot_seq) = Self::dot_delimited_id_sequence(exp) else {
            return Ok(None);
        };
        let mut scope = Some(verifier.scope());
        while let Some(scope1) = scope {
            let open_ns_set = scope1.concat_open_ns_set_of_scope_chain();
            let mut r: Option<Thingy> = None;
            for import in scope1.import_list().iter() {
                if let Some(r1) = Self::import_shadowing_package_name(verifier, &open_ns_set, &dot_seq, &import, &member_exp.identifier.location)? {
                    if r.is_some() && r.as_ref().unwrap() != &r1 {
                        verifier.add_verify_error(&member_exp.identifier.location, FxDiagnosticKind::AmbiguousReference, diagarg![dot_seq.last().unwrap().clone()]);
                        return Ok(None);
                    }
                    r = Some(r1);
                }
            }
            if let Some(r) = r {
                return Ok(Some(r));
            }
            scope = scope1.parent();
        }
        Ok(None)
    }

    fn import_shadowing_package_name(verifier: &mut Subverifier, open_ns_set: &SharedArray<Thingy>, dot_seq: &Vec<String>, import: &Thingy, location: &Location) -> Result<Option<Thingy>, DeferError> {
        if import.is::<PackageWildcardImport>() {
            if &dot_seq[0..(dot_seq.len() - 1)] != &import.package().fully_qualified_name_list() {
                return Ok(None);
            }
            match PropertyLookup(&verifier.host).lookup_in_object(&import.package(), &open_ns_set, None, &PropertyLookupKey::LocalName(dot_seq.last().unwrap().clone())) {
                Ok(Some(r)) => {
                    Unused(&verifier.host).mark_used(import);
                    return Ok(Some(r));
                },
                Ok(None) => {
                    return Ok(None);
                },
                Err(PropertyLookupError::AmbiguousReference(name)) => {
                    verifier.add_verify_error(&location, FxDiagnosticKind::AmbiguousReference, diagarg![name.clone()]);
                    return Ok(Some(verifier.host.invalidation_thingy()));
                },
                Err(PropertyLookupError::Defer) => {
                    return Err(DeferError());
                },
                Err(_) => {
                    panic!();
                },
            }
        } else if import.is::<PackageRecursiveImport>() {
            if &dot_seq[0..(dot_seq.len() - 1)] != &import.package().fully_qualified_name_list() {
                return Ok(None);
            }
            match PropertyLookup(&verifier.host).lookup_in_package_recursive(&import.package(), &open_ns_set, None, &PropertyLookupKey::LocalName(dot_seq.last().unwrap().clone())) {
                Ok(Some(r)) => {
                    Unused(&verifier.host).mark_used(import);
                    return Ok(Some(r));
                },
                Ok(None) => {
                    return Ok(None);
                },
                Err(PropertyLookupError::AmbiguousReference(name)) => {
                    verifier.add_verify_error(&location, FxDiagnosticKind::AmbiguousReference, diagarg![name.clone()]);
                    return Ok(Some(verifier.host.invalidation_thingy()));
                },
                Err(PropertyLookupError::Defer) => {
                    return Err(DeferError());
                },
                Err(_) => {
                    panic!();
                },
            }
        } else {
            assert!(import.is::<PackagePropertyImport>());
            if &dot_seq[0..(dot_seq.len() - 1)] != &import.property().parent().unwrap().fully_qualified_name_list()
            || dot_seq.last().unwrap() != &import.property().name().local_name()
            {
                return Ok(None);
            }
            Unused(&verifier.host).mark_used(import);
            Ok(Some(import.property().resolve_alias().wrap_property_reference(&verifier.host)?))
        }
    }

    fn dot_delimited_id_sequence(exp: &Rc<Expression>) -> Option<Vec<String>> {
        match exp.as_ref() {
            Expression::QualifiedIdentifier(id) => {
                id.to_identifier_name().map(|name| vec![name.0.clone()])
            },
            Expression::Member(m) => {
                let mut seq = Self::dot_delimited_id_sequence(&m.base)?;
                seq.push(m.identifier.to_identifier_name()?.0.clone());
                Some(seq)
            },
            _ => None,
        }
    }

    pub fn verify_computed_member_exp(verifier: &mut Subverifier, member_exp: &ComputedMemberExpression, context: &VerifierExpressionContext) -> Result<Option<Thingy>, DeferError> {
        let Some(base) = verifier.verify_expression(&member_exp.base, &default())? else {
            verifier.verify_expression(&member_exp.key, &default())?;
            return Ok(None);
        };

        let Some(key) = verifier.verify_expression(&member_exp.key, &default())? else {
            return Ok(None);
        };

        let open_ns_set = verifier.scope().concat_open_ns_set_of_scope_chain();
        let r = PropertyLookup(&verifier.host).lookup_in_object(&base, &open_ns_set, None, &PropertyLookupKey::Computed(key.clone()));
        if r.is_err() {
            match r.unwrap_err() {
                PropertyLookupError::AmbiguousReference(_) => {
                    panic!();
                },
                PropertyLookupError::Defer => {
                    return Err(DeferError());
                },
                PropertyLookupError::VoidBase => {
                    verifier.add_verify_error(&member_exp.key.location(), FxDiagnosticKind::AccessOfVoid, diagarg![]);
                    return Ok(None);
                },
                PropertyLookupError::NullableObject { .. } => {
                    verifier.add_verify_error(&member_exp.key.location(), FxDiagnosticKind::AccessOfNullable, diagarg![]);
                    return Ok(None);
                },
            }
        }
        let r = r.unwrap();
        if r.is_none() {
            panic!();
        }
        let r = r.unwrap();

        // No need to mark local capture for the property operator.
        // verifier.detect_local_capture(&r);

        // Post-processing
        verifier.reference_post_processing(r, context)
    }

    pub fn verify_descendants_exp(verifier: &mut Subverifier, desc_exp: &DescendantsExpression) -> Result<Option<Thingy>, DeferError> {
        let Some(base) = verifier.verify_expression(&desc_exp.base, &default())? else {
            Self::verify_qualified_identifier(verifier, &desc_exp.identifier)?;
            return Ok(None);
        };

        Self::verify_qualified_identifier(verifier, &desc_exp.identifier)?;

        let base_st = base.static_type(&verifier.host);
        let base_st_esc = base_st.escape_of_non_nullable();

        if ![verifier.host.any_type(),
            verifier.host.object_type().defer()?,
            verifier.host.xml_type().defer()?,
            verifier.host.xml_list_type().defer()?].contains(&base_st_esc) {
            verifier.add_verify_error(&desc_exp.identifier.location, FxDiagnosticKind::InapplicableDescendants, diagarg![base_st]);
            return Ok(None);
        }

        if [verifier.host.any_type(), verifier.host.object_type()].contains(&base_st_esc) {
            return Ok(Some(verifier.host.factory().create_value(&verifier.host.any_type())));
        }

        Ok(Some(verifier.host.factory().create_value(&verifier.host.xml_list_type())))
    }

    pub fn verify_filter_exp(verifier: &mut Subverifier, filter_exp: &FilterExpression) -> Result<Option<Thingy>, DeferError> {
        let Some(base) = verifier.verify_expression(&filter_exp.base, &default())? else {
            let scope = verifier.host.factory().create_filter_scope(&verifier.host.invalidation_thingy());
            verifier.inherit_and_enter_scope(&scope);
            verifier.verify_expression(&filter_exp.test, &default())?;
            verifier.exit_scope();

            return Ok(None);
        };

        let scope = verifier.host.factory().create_filter_scope(&base);
        verifier.inherit_and_enter_scope(&scope);
        verifier.verify_expression(&filter_exp.test, &default())?;
        verifier.exit_scope();

        let base_st = base.static_type(&verifier.host);
        let base_st_esc = base_st.escape_of_non_nullable();

        if ![verifier.host.any_type(),
            verifier.host.object_type().defer()?,
            verifier.host.xml_type().defer()?,
            verifier.host.xml_list_type().defer()?].contains(&base_st_esc) {
            verifier.add_verify_error(&filter_exp.test.location(), FxDiagnosticKind::InapplicableFilter, diagarg![base_st]);
            return Ok(None);
        }

        if [verifier.host.any_type(), verifier.host.object_type()].contains(&base_st_esc) {
            return Ok(Some(verifier.host.factory().create_value(&verifier.host.any_type())));
        }

        Ok(Some(verifier.host.factory().create_value(&verifier.host.xml_list_type())))
    }

    pub fn verify_super_exp(verifier: &mut Subverifier, super_exp: &SuperExpression) -> Result<Option<Thingy>, DeferError> {
        let Some(act) = verifier.scope().search_activation() else {
            if let Some(object) = super_exp.object.as_ref() {
                for obj in object {
                    verifier.verify_expression(obj, &default())?;
                }
            }
            verifier.add_verify_error(&super_exp.location, FxDiagnosticKind::ASuperExpCanBeUsedOnlyIn, diagarg![]);
            return Ok(None);
        };

        let Some(this) = act.this() else {
            if let Some(object) = super_exp.object.as_ref() {
                for obj in object {
                    verifier.verify_expression(obj, &default())?;
                }
            }
            verifier.add_verify_error(&super_exp.location, FxDiagnosticKind::ASuperExpCanBeUsedOnlyIn, diagarg![]);
            return Ok(None);
        };

        // In the future, provided interface methods could be supported, so
        // come with a pre-check.
        let this_st = this.static_type(&verifier.host);
        if this_st.is_interface_type_possibly_after_sub() {
            if let Some(object) = super_exp.object.as_ref() {
                for obj in object {
                    verifier.verify_expression(obj, &default())?;
                }
            }
            verifier.add_verify_error(&super_exp.location, FxDiagnosticKind::ASuperExpCanBeUsedOnlyIn, diagarg![]);
            return Ok(None);
        }


        let Some(limit) = this_st.extends_class(&verifier.host) else {
            if let Some(object) = super_exp.object.as_ref() {
                for obj in object {
                    verifier.verify_expression(obj, &default())?;
                }
            }
            verifier.add_verify_error(&super_exp.location, FxDiagnosticKind::ASuperExpCanOnlyBeUsedInSubclasses, diagarg![]);
            return Ok(None);
        };

        limit.defer()?;

        if let Some(object) = super_exp.object.as_ref() {
            if !object.is_empty() {
                for obj in &object[..(object.len() - 1)] {
                    verifier.verify_expression(obj, &default())?;
                }
                verifier.imp_coerce_exp(object.last().unwrap(), &limit)?;
            }
        }

        Ok(Some(verifier.host.factory().create_value(&limit)))
    }

    pub fn verify_call_exp(verifier: &mut Subverifier, exp: &CallExpression) -> Result<Option<Thingy>, DeferError> {
        let Some(base) = verifier.verify_expression(&exp.base, &default())? else {
            for arg in &exp.arguments {
                verifier.verify_expression(arg, &default())?;
            }
            return Ok(None);
        };

        // Type cast or new Array
        if let Some(base_type) = base.as_type() {
            let array_type = verifier.host.array_type().defer()?;
            // new Array
            if base_type == array_type || base_type.type_after_sub_has_origin(&array_type) {
                for arg in &exp.arguments {
                    verifier.verify_expression(arg, &default())?;
                }
                verifier.add_warning(&exp.base.location(), FxDiagnosticKind::CallOnArrayType, diagarg![]);
                return Ok(Some(verifier.host.factory().create_value(&array_type)));
            // Type cast
            } else {
                let mut first = true;
                for arg in &exp.arguments {
                    if first {
                        verifier.verify_expression(arg, &VerifierExpressionContext {
                            context_type: Some(base_type.clone()),
                            ..default()
                        })?;
                    } else {
                        verifier.verify_expression(arg, &default())?;
                    }
                    first = false;
                }
                if exp.arguments.len() < 1 {
                    verifier.add_verify_error(&exp.base.location(), FxDiagnosticKind::IncorrectNumArguments, diagarg!["1".to_string()]);
                } else if exp.arguments.len() > 1 {
                    verifier.add_verify_error(&exp.base.location(), FxDiagnosticKind::IncorrectNumArgumentsNoMoreThan, diagarg!["1".to_string()]);
                }
                return Ok(Some(verifier.host.factory().create_value(&base_type)));
            }
        }

        if base.is::<FixtureReferenceValue>() && base.property().is::<MethodSlot>() {
            let sig = base.property().signature(&verifier.host).defer()?;
            match ArgumentsSubverifier::verify(verifier, &exp.arguments, &sig) {
                Ok(_) => {},
                Err(VerifierArgumentsError::Defer) => {
                    return Err(DeferError());
                },
                Err(VerifierArgumentsError::Expected(n)) => {
                    verifier.add_verify_error(&exp.base.location(), FxDiagnosticKind::IncorrectNumArguments, diagarg![n.to_string()]);
                },
                Err(VerifierArgumentsError::ExpectedNoMoreThan(n)) => {
                    verifier.add_verify_error(&exp.base.location(), FxDiagnosticKind::IncorrectNumArgumentsNoMoreThan, diagarg![n.to_string()]);
                },
            }
            return Ok(Some(verifier.host.factory().create_value(&sig.result_type())));
        }

        let base_st = base.static_type(&verifier.host);
        let base_st_esc = base_st.escape_of_non_nullable();

        if base_st_esc.is::<FunctionType>() {
            let sig = base_st_esc;
            match ArgumentsSubverifier::verify(verifier, &exp.arguments, &sig) {
                Ok(_) => {},
                Err(VerifierArgumentsError::Defer) => {
                    return Err(DeferError());
                },
                Err(VerifierArgumentsError::Expected(n)) => {
                    verifier.add_verify_error(&exp.base.location(), FxDiagnosticKind::IncorrectNumArguments, diagarg![n.to_string()]);
                },
                Err(VerifierArgumentsError::ExpectedNoMoreThan(n)) => {
                    verifier.add_verify_error(&exp.base.location(), FxDiagnosticKind::IncorrectNumArgumentsNoMoreThan, diagarg![n.to_string()]);
                },
            }
            return Ok(Some(verifier.host.factory().create_value(&sig.result_type())));
        }

        for arg in &exp.arguments {
            verifier.verify_expression(arg, &default())?;
        }

        if ![verifier.host.any_type(), verifier.host.object_type().defer()?, verifier.host.function_type().defer()?].contains(&base_st_esc) {
            verifier.add_verify_error(&exp.base.location(), FxDiagnosticKind::CallOnNonFunction, diagarg![]);
            return Ok(None);
        }

        Ok(Some(verifier.host.factory().create_value(&verifier.host.any_type())))
    }

    pub fn verify_apply_types_exp(verifier: &mut Subverifier, exp: &ExpressionWithTypeArguments) -> Result<Option<Thingy>, DeferError> {
        let Some(base) = verifier.verify_expression(&exp.base, &VerifierExpressionContext {
            followed_by_type_arguments: true,
            ..default()
        })? else {
            for arg in &exp.arguments {
                verifier.verify_type_expression(arg)?;
            }
            return Ok(None);
        };

        // Ensure base is a type
        let Ok(base) = base.expect_type() else {
            for arg in &exp.arguments {
                verifier.verify_type_expression(arg)?;
            }
            verifier.add_verify_error(&exp.base.location(), FxDiagnosticKind::EntityIsNotAType, diagarg![]);
            return Ok(None);
        };

        // Ensure type is parameterized
        if !((base.is::<ClassType>() || base.is::<InterfaceType>()) && base.type_params().is_some()) {
            for arg in &exp.arguments {
                verifier.verify_type_expression(arg)?;
            }
            verifier.add_verify_error(&exp.base.location(), FxDiagnosticKind::NonParameterizedType, diagarg![]);
            return Ok(None);
        }

        let mut resolvee_args: SharedArray<Thingy> = shared_array![];
        let mut valid = true;

        for arg in &exp.arguments {
            if let Some(t) = verifier.verify_type_expression(arg)? {
                resolvee_args.push(t);
            } else {
                resolvee_args.push(verifier.host.invalidation_thingy());
                valid = false;
            }
        }

        let type_params = base.type_params().unwrap();

        if resolvee_args.length() < type_params.length() {
            verifier.add_verify_error(&exp.base.location(), FxDiagnosticKind::IncorrectNumArguments, diagarg![type_params.length().to_string()]);
            return Ok(None);
        } else if resolvee_args.length() > type_params.length() {
            verifier.add_verify_error(&exp.base.location(), FxDiagnosticKind::IncorrectNumArgumentsNoMoreThan, diagarg![type_params.length().to_string()]);
            return Ok(None);
        }

        if !valid {
            return Ok(None);
        }

        Ok(Some(verifier.host.factory().create_type_after_substitution(&base, &resolvee_args).wrap_property_reference(&verifier.host)?))
    }

    pub fn verify_unary_exp(verifier: &mut Subverifier, exp: &UnaryExpression) -> Result<Option<Thingy>, DeferError> {
        if exp.operator == Operator::Await {
            let Some(val) = verifier.verify_expression(&exp.expression, &default())? else {
                return Ok(None);
            };

            let val_st = val.static_type(&verifier.host);
            let Some(result_type) = val_st.escape_of_non_nullable().promise_result_type(&verifier.host)? else {
                verifier.add_verify_error(&exp.location, FxDiagnosticKind::AwaitOperandMustBeAPromise, diagarg![]);
                return Ok(None);
            };

            return Ok(Some(verifier.host.factory().create_value(&result_type)));
        }

        let update_ops = [Operator::PreIncrement, Operator::PreDecrement, Operator::PostIncrement, Operator::PostDecrement];
        let rw_mode = if exp.operator == Operator::Delete {
            VerifyMode::Delete
        } else if update_ops.contains(&exp.operator) {
            VerifyMode::Write
        } else {
            VerifyMode::Read
        };
        let Some(val) = verifier.verify_expression(&exp.expression, &VerifierExpressionContext {
            preceded_by_negative: exp.operator == Operator::Negative,
            mode: rw_mode,
            ..default()
        })? else {
            return Ok(None);
        };

        let val_st = val.static_type(&verifier.host);
        
        match exp.operator {
            Operator::PreIncrement |
            Operator::PreDecrement |
            Operator::PostIncrement |
            Operator::PostDecrement => {
                if !verifier.host.numeric_types()?.contains(&val_st) {
                    verifier.add_verify_error(&exp.expression.location(), FxDiagnosticKind::OperandMustBeNumber, diagarg![]);
                } else if val.write_only(&verifier.host) {
                    verifier.add_verify_error(&exp.expression.location(), FxDiagnosticKind::EntityIsWriteOnly, diagarg![]);
                }
                Ok(Some(verifier.host.factory().create_value(&val_st)))
            },
            Operator::NonNull => {
                if val_st.includes_undefined(&verifier.host)? || val_st.includes_null(&verifier.host)? {
                    let non_null_t = verifier.host.factory().create_non_nullable_type(&val_st);
                    Ok(Some(verifier.host.factory().create_value(&non_null_t)))
                } else {
                    verifier.add_warning(&exp.expression.location(), FxDiagnosticKind::ReferenceIsAlreadyNonNullable, diagarg![]);
                    Ok(Some(verifier.host.factory().create_value(&val_st)))
                }
            },
            Operator::Delete => {
                Ok(Some(verifier.host.factory().create_value(&verifier.host.boolean_type().defer()?)))
            },
            Operator::Void => {
                Ok(Some(verifier.host.factory().create_undefined_constant(&verifier.host.any_type())))
            },
            Operator::Typeof => {
                Ok(Some(verifier.host.factory().create_value(&verifier.host.string_type().defer()?)))
            },
            Operator::Yield => {
                verifier.add_verify_error(&exp.location, FxDiagnosticKind::YieldIsNotSupported, diagarg![]);
                Ok(None)
            },
            Operator::Positive => {
                let val_st_esc = val_st.escape_of_non_nullable();
                if !([verifier.host.any_type(), verifier.host.object_type().defer()?].contains(&val_st_esc) || verifier.host.numeric_types()?.contains(&val_st)) {
                    verifier.add_verify_error(&exp.expression.location(), FxDiagnosticKind::OperandMustBeNumber, diagarg![]);
                    return Ok(None);
                }
                if val.is::<NumberConstant>() {
                    return Ok(Some(val.clone()));
                }
                Ok(Some(verifier.host.factory().create_value(&val_st)))
            },
            Operator::Negative => {
                let val_st_esc = val_st.escape_of_non_nullable();
                if !([verifier.host.any_type(), verifier.host.object_type().defer()?].contains(&val_st_esc) || verifier.host.numeric_types()?.contains(&val_st)) {
                    verifier.add_verify_error(&exp.expression.location(), FxDiagnosticKind::OperandMustBeNumber, diagarg![]);
                    return Ok(None);
                }
                if val.is::<NumberConstant>() {
                    // Numeric literal has already been negated.
                    if matches!(exp.expression.as_ref(), Expression::NumericLiteral(_)) {
                        return Ok(Some(val.clone()));
                    }
                    return Ok(Some(verifier.host.factory().create_number_constant(-val.number_value(), &val_st)));
                }
                Ok(Some(verifier.host.factory().create_value(&val_st)))
            },
            Operator::BitwiseNot => {
                let val_st_esc = val_st.escape_of_non_nullable();
                if !([verifier.host.any_type(), verifier.host.object_type().defer()?].contains(&val_st_esc) || verifier.host.numeric_types()?.contains(&val_st)) {
                    verifier.add_verify_error(&exp.expression.location(), FxDiagnosticKind::OperandMustBeNumber, diagarg![]);
                    return Ok(None);
                }
                if val.is::<NumberConstant>() {
                    return Ok(Some(verifier.host.factory().create_number_constant(val.number_value().bitwise_not(), &val_st)));
                }
                Ok(Some(verifier.host.factory().create_value(&val_st)))
            },
            _ => {
                panic!();
            },
        }
    }
}