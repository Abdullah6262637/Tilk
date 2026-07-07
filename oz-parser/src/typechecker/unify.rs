use super::inference::TypeChecker;
use super::types::{Scheme, Type, TypeEnv};
use std::collections::{HashMap, HashSet};

impl TypeChecker {
    pub fn resolve(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(id) => {
                if let Some(resolved) = self.substitutions.get(id) {
                    self.resolve(resolved)
                } else {
                    ty.clone()
                }
            }
            Type::Array(inner) => Type::Array(Box::new(self.resolve(inner))),
            Type::Map(inner) => Type::Map(Box::new(self.resolve(inner))),
            Type::Task(inner) => Type::Task(Box::new(self.resolve(inner))),
            Type::Channel(inner) => Type::Channel(Box::new(self.resolve(inner))),
            Type::Option(inner) => Type::Option(Box::new(self.resolve(inner))),
            Type::Function { params, ret } => Type::Function {
                params: params.iter().map(|t| self.resolve(t)).collect(),
                ret: Box::new(self.resolve(ret)),
            },
            _ => ty.clone(),
        }
    }

    pub fn occurs_in(&self, var_id: usize, ty: &Type) -> bool {
        let resolved = self.resolve(ty);
        match resolved {
            Type::Var(id) => var_id == id,
            Type::Array(inner)
            | Type::Map(inner)
            | Type::Task(inner)
            | Type::Channel(inner)
            | Type::Option(inner) => self.occurs_in(var_id, &inner),
            Type::Function { params, ret } => {
                params.iter().any(|p| self.occurs_in(var_id, p)) || self.occurs_in(var_id, &ret)
            }
            _ => false,
        }
    }

    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<(), super::types::TypeError> {
        let t1 = self.resolve(t1);
        let t2 = self.resolve(t2);
        if t1 == t2 {
            return Ok(());
        }
        match (&t1, &t2) {
            (Type::Var(id1), _) => {
                if self.occurs_in(*id1, &t2) {
                    return Err(super::types::TypeError::new(
                        "Tip Hatası: Sonsuz tip (occurs check)",
                    ));
                }
                self.substitutions.insert(*id1, t2.clone());
                Ok(())
            }
            (_, Type::Var(id2)) => {
                if self.occurs_in(*id2, &t1) {
                    return Err(super::types::TypeError::new(
                        "Tip Hatası: Sonsuz tip (occurs check)",
                    ));
                }
                self.substitutions.insert(*id2, t1.clone());
                Ok(())
            }
            (Type::Array(inner1), Type::Array(inner2)) => self.unify(inner1, inner2),
            (Type::Map(inner1), Type::Map(inner2)) => self.unify(inner1, inner2),
            (Type::Task(inner1), Type::Task(inner2)) => self.unify(inner1, inner2),
            (Type::Channel(inner1), Type::Channel(inner2)) => self.unify(inner1, inner2),
            (Type::Option(inner1), Type::Option(inner2)) => self.unify(inner1, inner2),
            (Type::Option(_), Type::Bos) => Ok(()),
            (Type::Bos, Type::Option(_)) => Ok(()),
            (Type::Option(inner1), other) => self.unify(inner1, other),
            (other, Type::Option(inner2)) => self.unify(other, inner2),
            (
                Type::Function {
                    params: p1,
                    ret: r1,
                },
                Type::Function {
                    params: p2,
                    ret: r2,
                },
            ) => {
                if p1.len() != p2.len() {
                    return Err(super::types::TypeError::new(
                        "Tip Hatası: Fonksiyon argüman sayısı uyuşmuyor",
                    ));
                }
                for (p1_ty, p2_ty) in p1.iter().zip(p2.iter()) {
                    self.unify(p1_ty, p2_ty)?;
                }
                self.unify(r1, r2)
            }
            _ => Err(super::types::TypeError::new(format!(
                "Tip Hatası: {:?} ile {:?} tipleri birleştirilemiyor",
                t1, t2
            ))
            .with_expected(t1.clone())
            .with_found(t2.clone())),
        }
    }

    pub fn free_vars(&self, ty: &Type) -> HashSet<usize> {
        let resolved = self.resolve(ty);
        let mut vars = HashSet::new();
        self.collect_free_vars(&resolved, &mut vars);
        vars
    }

    pub fn collect_free_vars(&self, ty: &Type, vars: &mut HashSet<usize>) {
        match ty {
            Type::Var(id) => {
                if let Some(substituted) = self.substitutions.get(id) {
                    self.collect_free_vars(substituted, vars);
                } else {
                    vars.insert(*id);
                }
            }
            Type::Array(inner)
            | Type::Map(inner)
            | Type::Task(inner)
            | Type::Channel(inner)
            | Type::Option(inner) => {
                self.collect_free_vars(inner, vars);
            }
            Type::Function { params, ret } => {
                for p in params {
                    self.collect_free_vars(p, vars);
                }
                self.collect_free_vars(ret, vars);
            }
            _ => {}
        }
    }

    pub fn free_vars_env(&self, env: &TypeEnv) -> HashSet<usize> {
        let mut vars = HashSet::new();
        let mut current = Some(env);
        while let Some(e) = current {
            for scheme in e.bindings.values() {
                let scheme_free = self.free_vars(&scheme.ty);
                let quantified: HashSet<usize> = scheme.vars.iter().cloned().collect();
                let actual_free: HashSet<usize> =
                    scheme_free.difference(&quantified).cloned().collect();
                vars.extend(actual_free);
            }
            current = e.parent.as_deref();
        }
        vars
    }

    pub fn generalize(&self, ty: &Type, env: &TypeEnv) -> Scheme {
        let ty_free = self.free_vars(ty);
        let env_free = self.free_vars_env(env);
        let diff: Vec<usize> = ty_free.difference(&env_free).cloned().collect();
        Scheme {
            vars: diff,
            ty: ty.clone(),
        }
    }

    pub fn instantiate(&mut self, scheme: &Scheme) -> Type {
        let mut mapping = HashMap::new();
        for var_id in &scheme.vars {
            let fresh = self.new_var();
            mapping.insert(*var_id, Type::Var(fresh));
        }

        fn subst(ty: &Type, mapping: &HashMap<usize, Type>) -> Type {
            match ty {
                Type::Var(id) => {
                    if let Some(replacement) = mapping.get(id) {
                        replacement.clone()
                    } else {
                        ty.clone()
                    }
                }
                Type::Array(inner) => Type::Array(Box::new(subst(inner, mapping))),
                Type::Map(inner) => Type::Map(Box::new(subst(inner, mapping))),
                Type::Task(inner) => Type::Task(Box::new(subst(inner, mapping))),
                Type::Channel(inner) => Type::Channel(Box::new(subst(inner, mapping))),
                Type::Option(inner) => Type::Option(Box::new(subst(inner, mapping))),
                Type::Function { params, ret } => Type::Function {
                    params: params.iter().map(|p| subst(p, mapping)).collect(),
                    ret: Box::new(subst(ret, mapping)),
                },
                _ => ty.clone(),
            }
        }

        subst(&scheme.ty, &mapping)
    }
}
