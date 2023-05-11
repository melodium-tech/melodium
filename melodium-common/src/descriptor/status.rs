#[derive(Debug, Clone)]
pub enum Status<S, F, E> {
    Success { success: S, errors: Vec<E> },
    Failure { failure: F, errors: Vec<E> },
}

impl<S, F, E> Status<S, F, E> {
    pub fn new_success(success: S) -> Self {
        Self::Success {
            success,
            errors: Vec::new(),
        }
    }

    pub fn new_failure(failure: F) -> Self {
        Self::Failure {
            failure,
            errors: Vec::new(),
        }
    }

    pub const fn success(&self) -> Option<&S> {
        match self {
            Status::Success { success, errors: _ } => Some(success),
            Status::Failure {
                failure: _,
                errors: _,
            } => None,
        }
    }

    pub fn success_mut(&mut self) -> Option<&mut S> {
        match self {
            Status::Success { success, errors: _ } => Some(success),
            Status::Failure {
                failure: _,
                errors: _,
            } => None,
        }
    }

    pub fn is_success(&self) -> bool {
        match self {
            Status::Success {
                success: _,
                errors: _,
            } => true,
            Status::Failure {
                failure: _,
                errors: _,
            } => false,
        }
    }

    pub const fn failure(&self) -> Option<&F> {
        match self {
            Status::Success {
                success: _,
                errors: _,
            } => None,
            Status::Failure { failure, errors: _ } => Some(failure),
        }
    }

    pub fn failure_mut(&mut self) -> Option<&mut F> {
        match self {
            Status::Success {
                success: _,
                errors: _,
            } => None,
            Status::Failure { failure, errors: _ } => Some(failure),
        }
    }

    pub fn is_failure(&self) -> bool {
        match self {
            Status::Success {
                success: _,
                errors: _,
            } => false,
            Status::Failure {
                failure: _,
                errors: _,
            } => true,
        }
    }

    pub const fn errors(&self) -> &Vec<E> {
        match self {
            Status::Success { success: _, errors } => errors,
            Status::Failure { failure: _, errors } => errors,
        }
    }

    pub fn errors_mut(&mut self) -> &mut Vec<E> {
        match self {
            Status::Success { success: _, errors } => errors,
            Status::Failure { failure: _, errors } => errors,
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors().is_empty()
    }

    pub const fn as_result(&self) -> Result<&S, &F> {
        match self {
            Status::Success { success, errors: _ } => Ok(&success),
            Status::Failure { failure, errors: _ } => Err(&failure),
        }
    }

    pub fn into_result(self) -> Result<S, F> {
        match self {
            Status::Success { success, errors: _ } => Ok(success),
            Status::Failure { failure, errors: _ } => Err(failure),
        }
    }

    pub fn merge<T>(&mut self, status: Status<T, F, E>) -> Option<T> {
        match status {
            Status::Success {
                success,
                mut errors,
            } => {
                self.errors_mut().append(&mut errors);
                Some(success)
            }
            Status::Failure {
                failure,
                mut errors,
            } => {
                self.errors_mut().append(&mut errors);
                let mut errors = Vec::new();
                errors.append(self.errors_mut());
                *self = Self::Failure { failure, errors };
                None
            }
        }
    }

    pub fn and<T>(self, mut status: Status<T, F, E>) -> Status<T, F, E> {
        match self {
            Status::Success {
                success: _,
                errors: mut errors_self,
            } => match status {
                Status::Success {
                    success,
                    mut errors,
                } => {
                    errors_self.append(&mut errors);
                    Status::Success {
                        success,
                        errors: errors_self,
                    }
                }
                Status::Failure {
                    failure,
                    mut errors,
                } => {
                    errors_self.append(&mut errors);
                    Status::Failure {
                        failure,
                        errors: errors_self,
                    }
                }
            },
            Status::Failure {
                failure: failure_self,
                errors: mut errors_self,
            } => {
                errors_self.append(status.errors_mut());
                match status {
                    Status::Success {
                        success: _,
                        errors: _,
                    } => Status::Failure {
                        failure: failure_self,
                        errors: errors_self,
                    },
                    Status::Failure { failure, errors: _ } => Status::Failure {
                        failure,
                        errors: errors_self,
                    },
                }
            }
        }
    }

    pub fn and_then<T, G>(self, op: G) -> Status<T, F, E>
    where
        G: FnOnce(S) -> Status<T, F, E>,
    {
        match self {
            Status::Success {
                success,
                errors: mut errors_self,
            } => {
                let other = op(success);
                match other {
                    Status::Success {
                        success,
                        mut errors,
                    } => {
                        errors_self.append(&mut errors);
                        Status::Success {
                            success,
                            errors: errors_self,
                        }
                    }
                    Status::Failure {
                        failure,
                        mut errors,
                    } => {
                        errors_self.append(&mut errors);
                        Status::Failure {
                            failure,
                            errors: errors_self,
                        }
                    }
                }
            }
            Status::Failure { failure, errors } => Status::Failure { failure, errors },
        }
    }

    pub fn from<T, G, H>(value: Status<T, G, H>) -> Self
    where
        S: From<T>,
        F: From<G>,
        E: From<H>,
    {
        match value {
            Status::Success { success, errors } => Status::Success {
                success: S::from(success),
                errors: errors.into_iter().map(|e| E::from(e)).collect(),
            },
            Status::Failure { failure, errors } => Status::Failure {
                failure: F::from(failure),
                errors: errors.into_iter().map(|e| E::from(e)).collect(),
            },
        }
    }
}

impl<S, E> Status<S, E, E> {
    pub fn convert_failure_errors<F, G>(self, op: G) -> Status<S, F, F>
    where
        G: Fn(E) -> F,
    {
        match self {
            Status::Success { success, errors } => Status::Success {
                success,
                errors: errors.into_iter().map(op).collect(),
            },
            Status::Failure { failure, errors } => Status::Failure {
                failure: op(failure),
                errors: errors.into_iter().map(op).collect(),
            },
        }
    }

    pub fn and_degrade_failure<T>(self, mut status: Status<T, E, E>) -> Status<T, E, E> {
        match self {
            Status::Success {
                success: _,
                errors: mut errors_self,
            } => match status {
                Status::Success {
                    success,
                    mut errors,
                } => {
                    errors_self.append(&mut errors);
                    Status::Success {
                        success,
                        errors: errors_self,
                    }
                }
                Status::Failure {
                    failure,
                    mut errors,
                } => {
                    errors_self.append(&mut errors);
                    Status::Failure {
                        failure,
                        errors: errors_self,
                    }
                }
            },
            Status::Failure {
                failure: failure_self,
                errors: mut errors_self,
            } => {
                errors_self.append(status.errors_mut());
                match status {
                    Status::Success {
                        success: _,
                        errors: _,
                    } => Status::Failure {
                        failure: failure_self,
                        errors: errors_self,
                    },
                    Status::Failure { failure, errors: _ } => {
                        errors_self.push(failure_self);
                        Status::Failure {
                            failure,
                            errors: errors_self,
                        }
                    }
                }
            }
        }
    }

    pub fn merge_degrade_failure<T>(&mut self, status: Status<T, E, E>) -> Option<T>
    where
        E: Clone,
    {
        match status {
            Status::Success {
                success,
                mut errors,
            } => {
                self.errors_mut().append(&mut errors);
                Some(success)
            }
            Status::Failure {
                failure,
                mut errors,
            } => {
                match self {
                    Status::Success {
                        success: _,
                        errors: former_errors,
                    } => {
                        former_errors.append(&mut errors);
                        let mut self_errors = Vec::new();
                        self_errors.append(former_errors);
                        *self = Self::Failure {
                            failure,
                            errors: self_errors,
                        };
                    }
                    Status::Failure {
                        failure: former_failure,
                        errors: former_errors,
                    } => {
                        former_errors.push(former_failure.clone());
                        former_errors.append(&mut errors);
                        let mut self_errors = Vec::new();
                        self_errors.append(former_errors);
                        *self = Self::Failure {
                            failure,
                            errors: self_errors,
                        };
                    }
                }

                None
            }
        }
    }
}

impl<S, F, E> From<Result<S, F>> for Status<S, F, E> {
    fn from(value: Result<S, F>) -> Self {
        match value {
            Ok(success) => Self::new_success(success),
            Err(failure) => Self::new_failure(failure),
        }
    }
}
