use crate::ns::*;

pub struct ThingyFactory<'a>(pub(crate) &'a SemanticHost);

impl<'a> ThingyFactory<'a> {
    pub fn create_public_ns(&self, parent: Option<Thingy>) -> Thingy {
        SystemNamespace::new(&self.0.arena, SystemNamespaceKind::Public, parent).into()
    }

    pub fn create_private_ns(&self, parent: Option<Thingy>) -> Thingy {
        SystemNamespace::new(&self.0.arena, SystemNamespaceKind::Private, parent).into()
    }

    pub fn create_protected_ns(&self, parent: Option<Thingy>) -> Thingy {
        SystemNamespace::new(&self.0.arena, SystemNamespaceKind::Protected, parent).into()
    }

    pub fn create_static_protected_ns(&self, parent: Option<Thingy>) -> Thingy {
        SystemNamespace::new(&self.0.arena, SystemNamespaceKind::StaticProtected, parent).into()
    }

    pub fn create_internal_ns(&self, parent: Option<Thingy>) -> Thingy {
        SystemNamespace::new(&self.0.arena, SystemNamespaceKind::Internal, parent).into()
    }

    pub fn create_explicit_ns(&self, uri: String) -> Thingy {
        let mut mappings = self.0.explicit_namespaces.borrow_mut();
        if let Some(ns) = mappings.get(&uri) {
            return ns.clone();
        }
        let ns: Thingy = ExplicitNamespace::new(&self.0.arena, uri.clone()).into();
        mappings.insert(uri, ns.clone());
        ns
    }

    pub fn create_user_ns(&self, uri: String) -> Thingy {
        let mut mappings = self.0.user_namespaces.borrow_mut();
        if let Some(ns) = mappings.get(&uri) {
            return ns.clone();
        }
        let ns: Thingy = UserNamespace::new(&self.0.arena, uri.clone()).into();
        mappings.insert(uri, ns.clone());
        ns
    }

    pub fn create_qname(&self, namespace: &Thingy, local_name: String) -> QName {
        let mut ns_mappings = self.0.qnames.borrow_mut();
        if let Some(qn_mappings) = ns_mappings.get_mut(namespace) {
            if let Some(qn) = qn_mappings.get(&local_name) {
                return qn.clone();
            }
            let qn = QName(Rc::new(QName1 {
                m_namespace: namespace.clone(),
                m_local_name: local_name.clone(),
            }));
            qn_mappings.insert(local_name, qn.clone());
            return qn;
        }
        let qn = QName(Rc::new(QName1 {
            m_namespace: namespace.clone(),
            m_local_name: local_name.clone(),
        }));
        let mut qn_mappings = HashMap::new();
        qn_mappings.insert(local_name, qn.clone());
        ns_mappings.insert(namespace.clone(), qn_mappings);
        qn
    }

    pub fn create_ns_set(&self, list: SharedArray<Thingy>) -> Thingy {
        // Do not intern namespace sets for now.
        NamespaceSet::new(&self.0.arena, list).into()
    }

    /// Interns a package from a fully qualified name.
    ///
    /// # Example
    ///
    /// ```ignore
    /// assert_eq!(host.factory().create_package(["foo", "bar"]).fully_qualified_name(), "foo.bar");
    /// ```
    pub fn create_package<'b>(&self, name: impl IntoIterator<Item = &'b str>) -> Thingy {
        self.create_package_1(&name.into_iter().collect())
    }

    fn create_package_1(&self, name: &Vec<&str>) -> Thingy {
        let mut result: Thingy = self.0.top_level_package.clone();
        for name_1 in name {
            let name_1 = (*name_1).to_owned();
            let result_1 = result.subpackages().get(&name_1);
            if let Some(result_1) = result_1 {
                result = result_1;
            } else {
                let result_1 = Package::new(&self.0.arena, name_1.clone());
                result_1.set_parent(Some(result.clone().into()));

                // Assign namespaces
                result_1.set_public_ns(Some(self.create_public_ns(Some(result_1.clone().into()))));
                result_1.set_internal_ns(Some(self.create_internal_ns(Some(result_1.clone().into()))));

                result.subpackages().set(name_1, result_1.clone().into());
                result = result_1.into();
            }
        }
        result
    }

    pub fn create_alias(&self, name: QName, alias_of: Thingy) -> Thingy {
        Alias::new(&self.0.arena, name, alias_of).into()
    }

    pub fn create_class_type(&self, name: QName) -> Thingy {
        let r = ClassType::new(&self.0.arena, name);
        r.set_private_ns(Some(self.create_private_ns(Some(r.clone().into()))));
        r.set_protected_ns(Some(self.create_protected_ns(Some(r.clone().into()))));
        r.set_static_protected_ns(Some(self.create_static_protected_ns(Some(r.clone().into()))));
        r.into()
    }

    pub fn create_enum_type(&self, name: QName) -> Thingy {
        let r = EnumType::new(&self.0.arena, name);
        r.set_private_ns(Some(self.create_private_ns(Some(r.clone().into()))));
        r.into()
    }

    pub fn create_interface_type(&self, name: QName) -> Thingy {
        let r = InterfaceType::new(&self.0.arena, name);
        r.into()
    }

    /// Interns type after substitution.
    pub fn create_type_after_substitution(&self, origin: &Thingy, substitute_types: &SharedArray<Thingy>) -> Thingy {
        // Verify parameter count
        let params = origin.type_params().unwrap();
        let param_count = params.length();
        assert_eq!(substitute_types.length(), param_count);

        let mut tas_list = self.0.types_after_sub.borrow_mut();

        let mut list = tas_list.get(&origin);
        let empty_list = vec![];
        if list.is_none() {
            list = Some(&empty_list);
            tas_list.insert(origin.clone(), vec![]);
        }
        'tas: for tas in list.unwrap() {
            let mut substitute_types_1 = substitute_types.iter();
            let substitute_types_2 = tas.substitute_types();
            let mut substitute_types_2 = substitute_types_2.iter();
            while let Some(substitute_type_1) = substitute_types_1.next() {
                let substitute_type_2 = substitute_types_2.next().unwrap();
                if substitute_type_1 != substitute_type_2 {
                    continue 'tas;
                }
            }
            return tas.clone();
        }

        let tas = TypeAfterSubstitution::new(&self.0.arena, origin.clone(), substitute_types.clone());
        let list = tas_list.get_mut(&origin).unwrap();
        list.push(tas.clone().into());

        tas.into()
    }

    /// Interns a tuple type.
    pub fn create_tuple_type(&self, element_types: Vec<Thingy>) -> Thingy {
        let element_count = element_types.len();
        let mut tuple_types = self.0.tuple_types.borrow_mut();
        let mut collection = tuple_types.get_mut(&element_count);
        let mut empty_collection = vec![];
        if collection.is_none() {
            collection = Some(&mut empty_collection);
            tuple_types.insert(element_count, vec![]);
        }
        'tt: for tt in collection.unwrap() {
            let mut element_types_1 = element_types.iter();
            let element_types_2 = tt.element_types();
            let mut element_types_2 = element_types_2.iter();
            while let Some(e_1) = element_types_1.next() {
                let e_2 = element_types_2.next().unwrap();
                if e_1 != &e_2 {
                    continue 'tt;
                }
            }
            return tt.clone();
        }
        let tt = TupleType::new(&self.0.arena, SharedArray::from(element_types));

        let collection = tuple_types.get_mut(&element_count);
        collection.unwrap().push(tt.clone().into());

        tt.into()
    }

    /// Interns a function type.
    pub fn create_function_type(&self, params: Vec<Rc<SemanticFunctionTypeParameter>>, result_type: Thingy) -> Thingy {
        let param_count = params.len();
        let mut function_types = self.0.function_types.borrow_mut();
        let mut collection = function_types.get_mut(&param_count);
        let mut empty_collection = vec![];
        if collection.is_none() {
            collection = Some(&mut empty_collection);
            function_types.insert(params.len(), vec![]);
        }
        'ft: for ft in collection.unwrap() {
            if result_type != ft.result_type() {
                continue 'ft;
            }
            let mut params_1 = params.iter();
            let params_2 = ft.params();
            let mut params_2 = params_2.iter();
            while let Some(param_1) = params_1.next() {
                let param_2 = params_2.next().unwrap();
                if !(param_1.kind == param_2.kind && && param_1.static_type == &&param_2.static_type) {
                    continue 'ft;
                }
            }
            return ft.clone();
        }
        let ft = FunctionType::new(&self.0.arena, SharedArray::from(params), result_type);

        let collection = function_types.get_mut(&param_count);
        collection.unwrap().push(ft.clone().into());

        ft.into()
    }

    /// Interns a nullable type.
    pub fn create_nullable_type(&self, base: &Thingy) -> Thingy {
        if base == &self.0.any_type() {
            return base.clone();
        }
        let mut m = self.0.nullable_types.borrow_mut();
        let nt = m.get(base);
        if let Some(nt) = nt {
            return nt.clone();
        }
        let nt = NullableType::new(&self.0.arena, base.clone());
        m.insert(base.clone(), nt.clone().into());
        nt.into()
    }

    /// Interns a non nullable type.
    pub fn create_non_nullable_type(&self, base: &Thingy) -> Thingy {
        if base == &self.0.any_type() {
            return base.clone();
        }
        let mut m = self.0.non_nullable_types.borrow_mut();
        let nt = m.get(base);
        if let Some(nt) = nt {
            return nt.clone();
        }
        let nt = NonNullableType::new(&self.0.arena, base.clone());
        m.insert(base.clone(), nt.clone().into());
        nt.into()
    }

    pub fn create_type_parameter(&self, name: &QName) -> Thingy {
        TypeParameterType::new(&self.0.arena, name.clone()).into()
    }
}