//! Principals — Human vs Mind capability boundary.
//!
//! ## Design (revised)
//!
//! **SLEEP / SAVE must be available to Mind.**  
//! Without consolidation, intelligence cannot persist across boots or generations.
//! That is a no-brainer — auto-LTM is cortex-native (like sleep consolidation).
//!
//! **Identity is not a free rewrite.**  
//! Full `WRITE /home/identity` that *replaces* the self is Human-only (or rare).  
//! Mind may **DEFRAG** (reorganize/consolidate without erasing core self) and  
//! **APPEND session:** lines (episodic notes), never clobber `core:`.
//!
//! Biology:
//! - SLEEP/SAVE ≈ systems consolidation (must run without an operator present)
//! - identity `core:` ≈ stable autobiographical self
//! - DEFRAG ≈ memory reorganization / synaptic renormalization
//! - APPEND session ≈ new episodic trace, not a personality wipe

use alloc::string::String;

use crate::mind_abi::MotorVerb;
use crate::organs::OrganId;

/// Who is issuing motor intent.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Principal {
    /// Operator (desktop viz / keyboard) — can hard-rewrite identity if needed.
    Human,
    /// Living cortex via Mind ABI — may sleep/save; identity only via defrag/append.
    Mind,
}

impl Principal {
    pub fn label(self) -> &'static str {
        match self {
            Principal::Human => "human",
            Principal::Mind => "mind",
        }
    }

    pub fn from_str_name(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("human") || s.eq_ignore_ascii_case("operator") {
            Some(Principal::Human)
        } else if s.eq_ignore_ascii_case("mind") || s.eq_ignore_ascii_case("cortex") {
            Some(Principal::Mind)
        } else {
            None
        }
    }
}

/// Capability token for a principal.
#[derive(Copy, Clone, Debug)]
pub struct Capabilities {
    pub status_read: bool,
    pub attend: bool,
    pub express_repress: bool,
    pub write_organs: bool,
    pub write_home_knowledge: bool,
    pub write_home_tools: bool,
    pub write_home_genome: bool,
    /// Full clobber of /home/identity (dangerous — Human only).
    pub rewrite_home_identity: bool,
    /// Safe identity maintenance: DEFRAG + APPEND session.
    pub defrag_identity: bool,
    /// SLEEP/SAVE — required for LTM across boots (both principals).
    pub sleep_save: bool,
    pub init_sched: bool,
}

impl Capabilities {
    pub fn for_principal(p: Principal) -> Self {
        match p {
            Principal::Human => Capabilities {
                status_read: true,
                attend: true,
                express_repress: true,
                write_organs: true,
                write_home_knowledge: true,
                write_home_tools: true,
                write_home_genome: true,
                rewrite_home_identity: true,
                defrag_identity: true,
                sleep_save: true,
                init_sched: true,
            },
            Principal::Mind => Capabilities {
                status_read: true,
                attend: true,
                express_repress: true,
                write_organs: true,
                write_home_knowledge: true,
                write_home_tools: true,
                write_home_genome: true,
                rewrite_home_identity: false, // no personality wipe
                defrag_identity: true,        // consolidate only
                sleep_save: true,             // MUST persist intelligence
                init_sched: true,
            },
        }
    }
}

/// Authorize a motor verb for principal.
pub fn authorize(principal: Principal, verb: &MotorVerb) -> Result<(), String> {
    let c = Capabilities::for_principal(principal);
    match verb {
        MotorVerb::Status
        | MotorVerb::Report
        | MotorVerb::Organs
        | MotorVerb::Home
        | MotorVerb::Sched
        | MotorVerb::Scale
        | MotorVerb::Read(_)
        | MotorVerb::Wake => {
            if c.status_read {
                Ok(())
            } else {
                Err(String::from("cap_denied_read"))
            }
        }
        MotorVerb::Attend(_) => {
            if c.attend {
                Ok(())
            } else {
                Err(String::from("cap_denied_attend"))
            }
        }
        MotorVerb::Express | MotorVerb::Repress | MotorVerb::Breathe => {
            if c.express_repress {
                Ok(())
            } else {
                Err(String::from("cap_denied_modulate"))
            }
        }
        MotorVerb::Note(_) => {
            if c.write_organs {
                Ok(())
            } else {
                Err(String::from("cap_denied_note"))
            }
        }
        MotorVerb::Write { path, .. } => authorize_write(principal, path),
        MotorVerb::Append { path, .. } => authorize_append(principal, path),
        MotorVerb::Defrag { path } => authorize_defrag(principal, path),
        MotorVerb::Sleep | MotorVerb::Save => {
            // Both Human and Mind — LTM across boots is non-negotiable
            if c.sleep_save {
                Ok(())
            } else {
                Err(String::from("cap_denied_sleep"))
            }
        }
        MotorVerb::Init => {
            if c.init_sched {
                Ok(())
            } else {
                Err(String::from("cap_denied_init"))
            }
        }
    }
}

fn authorize_write(principal: Principal, path: &str) -> Result<(), String> {
    let c = Capabilities::for_principal(principal);
    let id = OrganId::from_path(path);
    match id {
        Some(OrganId::HomeIdentity) => {
            if c.rewrite_home_identity {
                Ok(())
            } else {
                Err(String::from(
                    "cap_denied_identity_rewrite — use MOTOR DEFRAG /home/identity or APPEND session:… (Mind cannot clobber self)",
                ))
            }
        }
        Some(OrganId::HomeGenome) => {
            if c.write_home_genome {
                Ok(())
            } else {
                Err(String::from("cap_denied_genome"))
            }
        }
        Some(OrganId::HomeKnowledge) => {
            if c.write_home_knowledge {
                Ok(())
            } else {
                Err(String::from("cap_denied_home_knowledge"))
            }
        }
        Some(OrganId::HomeTools) | Some(OrganId::HomeProposals) | Some(OrganId::HomeMedia) => {
            if c.write_home_tools {
                Ok(())
            } else {
                Err(String::from("cap_denied_home_tools"))
            }
        }
        Some(_) | None => {
            if c.write_organs {
                Ok(())
            } else {
                Err(String::from("cap_denied_write"))
            }
        }
    }
}

fn authorize_append(principal: Principal, path: &str) -> Result<(), String> {
    let c = Capabilities::for_principal(principal);
    if OrganId::from_path(path) == Some(OrganId::HomeIdentity) {
        if c.defrag_identity {
            Ok(())
        } else {
            Err(String::from("cap_denied_identity_append"))
        }
    } else {
        // append elsewhere = same as write for Mind/Human
        authorize_write(principal, path).or_else(|_| {
            if c.write_organs {
                Ok(())
            } else {
                Err(String::from("cap_denied_append"))
            }
        })
    }
}

fn authorize_defrag(principal: Principal, path: &str) -> Result<(), String> {
    let c = Capabilities::for_principal(principal);
    if OrganId::from_path(path) == Some(OrganId::HomeIdentity)
        || path.contains("home")
        || path.contains("knowledge")
    {
        if c.defrag_identity || c.write_home_knowledge {
            Ok(())
        } else {
            Err(String::from("cap_denied_defrag"))
        }
    } else if c.write_organs {
        Ok(())
    } else {
        Err(String::from("cap_denied_defrag"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mind_can_sleep_and_save() {
        assert!(authorize(Principal::Mind, &MotorVerb::Sleep).is_ok());
        assert!(authorize(Principal::Mind, &MotorVerb::Save).is_ok());
        assert!(authorize(Principal::Human, &MotorVerb::Sleep).is_ok());
    }

    #[test]
    fn mind_cannot_rewrite_identity_but_can_defrag() {
        let wipe = MotorVerb::Write {
            path: String::from("/home/identity"),
            data: String::from("hack"),
        };
        assert!(authorize(Principal::Mind, &wipe).is_err());
        assert!(authorize(Principal::Human, &wipe).is_ok());

        let defrag = MotorVerb::Defrag {
            path: String::from("/home/identity"),
        };
        assert!(authorize(Principal::Mind, &defrag).is_ok());

        let append = MotorVerb::Append {
            path: String::from("/home/identity"),
            data: String::from("session: gen=3 hero=x"),
        };
        assert!(authorize(Principal::Mind, &append).is_ok());
    }

    #[test]
    fn mind_can_write_knowledge() {
        let v = MotorVerb::Write {
            path: String::from("/home/knowledge"),
            data: String::from("fact"),
        };
        assert!(authorize(Principal::Mind, &v).is_ok());
    }
}
