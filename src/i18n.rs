// ── i18n (FR / EN) ────────────────────────────────────────────────────────────
// Simple const-key dictionary, mirroring the CVGenerator/GameTracker pattern.

#[cfg(target_arch = "wasm32")]
const LANG_KEY: &str = "sre_audit_lang";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Fr,
    En,
}

impl Lang {
    pub fn detect() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(s) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
                .and_then(|s| s.get_item(LANG_KEY).ok())
                .flatten()
            {
                return if s.starts_with("en") {
                    Lang::En
                } else {
                    Lang::Fr
                };
            }
            if let Some(nav) = web_sys::window().and_then(|w| w.navigator().language()) {
                if nav.starts_with("fr") {
                    return Lang::Fr;
                }
            }
            Lang::En
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Lang::Fr
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::En => Self::Fr,
            Self::Fr => Self::En,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::En => "EN",
            Self::Fr => "FR",
        }
    }

    pub fn persist(self) {
        let s = match self {
            Self::En => "en",
            Self::Fr => "fr",
        };
        #[cfg(target_arch = "wasm32")]
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            let _ = storage.set_item(LANG_KEY, s);
        }
        let _ = s;
    }
}

pub fn tr(key: &str, lang: Lang) -> String {
    let val = match lang {
        Lang::En => en(key),
        Lang::Fr => fr(key),
    };
    val.to_string()
}

fn en(key: &str) -> &str {
    match key {
        "nav_home" => "Home",
        "nav_matrix" => "Maturity Matrix",
        "nav_roadmap" => "Roadmap",
        "nav_backup" => "Backup",
        "nav_signout" => "Sign out",
        "theme_dark" => "Dark",
        "theme_light" => "Light",

        "login_title" => "SRE Audit - Restricted Access",
        "login_subtitle" => "Sign in with your Google account to access the SRE audit tools.",
        "login_button" => "Sign in with Google",
        "login_error" => "Google error",
        "login_config_error" => "GOOGLE_CLIENT_ID not configured at build time.",

        "denied_title" => "Access denied",
        "denied_body" => "Your Google account is not on the authorised list for this tool.",
        "denied_email" => "Signed in as",
        "denied_back" => "Try another account",
        "denied_nothing" => "You don't have access to the SRE audit tools.",

        "home_title" => "SRE Audit",
        "home_subtitle" => "Maturity matrix and transformation roadmap for your SRE journey.",
        "home_welcome" => "Welcome",
        "home_matrix_title" => "SRE Maturity Matrix",
        "home_matrix_desc" => "Assess 7 SRE principles against a 4-level maturity scale with per-row notes.",
        "home_matrix_btn" => "Open the matrix",
        "home_roadmap_title" => "Transformation Roadmap",
        "home_roadmap_desc" => "Short and long term action plans to move the organisation forward.",
        "home_roadmap_btn" => "Open the roadmap",
        "home_orgs" => "Audited organisations",
        "backup_title" => "Global backup",
        "backup_desc" => "Back up or restore every organisation (matrix and roadmap) in a single Google Drive file.",
        "org_add" => "Add organisation",
        "org_add_ph" => "New organisation name…",
        "org_name_label" => "Name",
        "org_current" => "Active",
        "org_delete" => "Delete organisation",

        "status_saved" => "✓ Saved locally",
        "status_loading" => "Loading…",
        "save" => "Save",
        "export_pdf" => "Export as PDF",
        "export_json" => "Export JSON",
        "import_json" => "Import JSON",
        "drive_upload" => "Backup to Drive",
        "drive_restore" => "Restore from Drive",
        "flash_saved" => "✓ Changes saved",
        "flash_loaded" => "✓ Loaded",
        "flash_drive_up" => "✓ Backed up to Drive",
        "flash_drive_down" => "✓ Restored from Drive",
        "flash_import_ok" => "✓ Imported",
        "flash_import_err" => "✗ Invalid JSON file",
        "error" => "Error",
        "print_placeholder" => "No particular comment.",

        "matrix_company" => "Organisation audited:",
        "matrix_company_ph" => "Enter the company name…",
        "matrix_desc" => "Cross-evaluation framework aligning Google SRE pillars with ITIL process modernisation (Incident, Problem, Change, Capacity) and Cloud Native practices.",
        "matrix_instructions" => "Click a level to position the organisation. Add field notes on the right. Use “Save” to keep your data between sessions.",
        "matrix_col1" => "SRE Principle & IT Process",
        "matrix_col2" => "Level 1: Reactive",
        "matrix_col3" => "Level 2: Organised",
        "matrix_col4" => "Level 3: Proactive",
        "matrix_col5" => "Level 4: SRE / Continuous",
        "matrix_comment" => "Notes & Nuances (Audit)",
        "matrix_aspect" => "Aspect",
        "matrix_title" => "SRE Maturity Matrix & Audit Notes",
        "matrix_comment_ph" => "Enter your remarks or level nuances here…",

        "roadmap_title" => "Institutionalised SRE Roadmap",
        "roadmap_meta" => "Focus: Tool convergence, delivery reliability & collaborative alignment",
        "roadmap_st_title" => "Short-Term Roadmap (3 Months: Rationalisation & Urgent Actions)",
        "roadmap_lt_title" => "Long-Term Roadmap (2-3 Years: Industrialisation & Unified Ecosystem)",
        "roadmap_st_h1" => "SRE Principle / ITIL Process",
        "roadmap_st_h2" => "Audit Finding (Origin)",
        "roadmap_st_h3" => "Immediate Priority Actions (M1 to M3)",
        "roadmap_st_h4" => "Deliverables & KPI (3M)",
        "roadmap_lt_h1" => "SRE Principle / ITIL Process",
        "roadmap_lt_h2" => "Consolidation Target (Year 1)",
        "roadmap_lt_h3" => "Major Transformation (Years 2 & 3)",
        "roadmap_lt_h4" => "Target Vision (Continuous SRE Level)",
        "roadmap_generate" => "Generate from matrix",
        "roadmap_gen_ok" => "Roadmap filled from the matrix (empty cells only).",
        "roadmap_gen_noselection" => "Select maturity levels in the matrix first.",
        "roadmap_gen_overwrite" => "Force regenerate",
        "roadmap_gen_forced" => "Roadmap regenerated from the matrix.",
        "roadmap_clear" => "Clear roadmap",
        "roadmap_clear_ok" => "Roadmap cleared.",
        "roadmap_undo" => "Undo",
        "roadmap_undo_ok" => "Changes undone.",
        "roadmap_confirm_overwrite" => "Regenerate all roadmap cells from the matrix? Edited content on selected rows will be overwritten.",
        "roadmap_confirm_clear" => "Clear the entire roadmap? All the cells will be emptied.",
        "roadmap_note_label" => "Audit note",
        "roadmap_tools_label" => "Involved tools",

        "drive_hint" => "Your data is stored locally in this browser. Use “Backup to Drive” to save to your own private Google Drive app folder.",

        _ => key,
    }
}

fn fr(key: &str) -> &str {
    match key {
        "nav_home" => "Accueil",
        "nav_matrix" => "Matrice de Maturité",
        "nav_roadmap" => "Roadmap",
        "nav_backup" => "Sauvegarde",
        "nav_signout" => "Se déconnecter",
        "theme_dark" => "Sombre",
        "theme_light" => "Clair",

        "login_title" => "SRE Audit - Accès Restreint",
        "login_subtitle" => "Connectez-vous avec votre compte Google pour accéder aux outils d'audit SRE.",
        "login_button" => "Se connecter avec Google",
        "login_error" => "Erreur Google",
        "login_config_error" => "GOOGLE_CLIENT_ID non configuré à la compilation.",

        "denied_title" => "Accès refusé",
        "denied_body" => "Votre compte Google n'est pas sur la liste des comptes autorisés pour cet outil.",
        "denied_email" => "Connecté en tant que",
        "denied_back" => "Essayer un autre compte",
        "denied_nothing" => "Vous n'avez pas accès aux outils d'audit SRE.",

        "home_title" => "SRE Audit",
        "home_subtitle" => "Matrice de maturité et feuille de route de transformation pour votre parcours SRE.",
        "home_welcome" => "Bienvenue",
        "home_matrix_title" => "Matrice de Maturité SRE",
        "home_matrix_desc" => "Évaluez 7 principes SRE sur une échelle de maturité à 4 niveaux, avec notes par ligne.",
        "home_matrix_btn" => "Ouvrir la matrice",
        "home_roadmap_title" => "Feuille de Route de Transformation",
        "home_roadmap_desc" => "Plans d'action court et long terme pour faire évoluer l'organisation.",
        "home_roadmap_btn" => "Ouvrir la roadmap",
        "home_orgs" => "Organisations auditées",
        "backup_title" => "Sauvegarde globale",
        "backup_desc" => "Sauvegardez ou restaurez toutes vos organisations (matrice et feuille de route) dans un seul fichier Google Drive.",
        "org_add" => "Ajouter une organisation",
        "org_add_ph" => "Nom de la nouvelle organisation…",
        "org_name_label" => "Nom",
        "org_current" => "Active",
        "org_delete" => "Supprimer l'organisation",

        "status_saved" => "✓ Enregistré localement",
        "status_loading" => "Chargement…",
        "save" => "Sauvegarder",
        "export_pdf" => "Exporter en PDF",
        "export_json" => "Exporter JSON",
        "import_json" => "Importer JSON",
        "drive_upload" => "Sauvegarder sur Drive",
        "drive_restore" => "Restaurer depuis Drive",
        "flash_saved" => "✓ Changements enregistrés",
        "flash_loaded" => "✓ Chargé",
        "flash_drive_up" => "✓ Sauvegardé sur Drive",
        "flash_drive_down" => "✓ Restauré depuis Drive",
        "flash_import_ok" => "✓ Importé",
        "flash_import_err" => "✗ Fichier JSON invalide",
        "error" => "Erreur",
        "print_placeholder" => "Aucun commentaire particulier.",

        "matrix_company" => "Organisation Auditée :",
        "matrix_company_ph" => "Saisir le nom de l'entreprise...",
        "matrix_desc" => "Framework d'évaluation croisée alignant les piliers Google SRE avec la modernisation des processus ITIL (Incident, Problème, Changement, Capacité) et les pratiques Cloud Native.",
        "matrix_instructions" => "💡 Mode d'emploi : Cliquez sur une cellule pour positionner l'organisation. Renseignez vos commentaires de terrain à droite. Utilisez \"Sauvegarder\" pour conserver vos données locales d'une session à l'autre.",
        "matrix_col1" => "Principe SRE & Processus IT",
        "matrix_col2" => "Niveau 1 : Réactif",
        "matrix_col3" => "Niveau 2 : Organisé",
        "matrix_col4" => "Niveau 3 : Proactif",
        "matrix_col5" => "Niveau 4 : SRE / Continu",
        "matrix_comment" => "Commentaires & Nuances (Audit)",
        "matrix_aspect" => "Aspect",
        "matrix_title" => "Matrice de Maturité SRE & Notes d'Audit",
        "matrix_comment_ph" => "Saisir vos remarques ou nuances de niveau ici...",
        "roadmap_title" => "Feuille de Route SRE Institutionnalisée",
        "roadmap_meta" => "Focalisation : Convergence des Outils, Fiabilisation du Delivery & Alignment Collaboratif",
        "roadmap_st_title" => "⏱️ Feuille de Route Court Terme (3 Mois : Rationalisation & Actions d'Urgence)",
        "roadmap_lt_title" => "🚀 Feuille de Route Long Terme (2-3 Ans : Industrialisation & Écosystème Unifié)",
        "roadmap_st_h1" => "Principe SRE / Processus ITIL",
        "roadmap_st_h2" => "Constat d'Audit (Origine)",
        "roadmap_st_h3" => "Actions Prioritaires Immédiates (M1 à M3)",
        "roadmap_st_h4" => "Livrables & KPI (3M)",
        "roadmap_lt_h1" => "Principe SRE / Processus ITIL",
        "roadmap_lt_h2" => "Cible de Consolidation (Année 1)",
        "roadmap_lt_h3" => "Transformation Majeure (Années 2 & 3)",
        "roadmap_lt_h4" => "Vision Target (Niveau SRE Continu)",
        "roadmap_generate" => "Générer depuis la matrice",
        "roadmap_gen_ok" => "Feuille de route remplie depuis la matrice (cellules vides uniquement).",
        "roadmap_gen_noselection" => "Sélectionnez d'abord des niveaux de maturité dans la matrice.",
        "roadmap_gen_overwrite" => "Régénérer",
        "roadmap_gen_forced" => "Feuille de route régénérée depuis la matrice.",
        "roadmap_clear" => "Vider la feuille de route",
        "roadmap_clear_ok" => "Feuille de route vidée.",
        "roadmap_undo" => "Annuler",
        "roadmap_undo_ok" => "Modifications annulées.",
        "roadmap_confirm_overwrite" => "Régénérer toute la feuille de route depuis la matrice ? Le contenu édité des lignes sélectionnées sera écrasé.",
        "roadmap_confirm_clear" => "Vider entièrement la feuille de route ? Toutes les cellules seront effacées.",
        "roadmap_note_label" => "Note d'audit",
        "roadmap_tools_label" => "Outils impliqués",

        "drive_hint" => "Vos données sont stockées localement dans ce navigateur. Utilisez \"Sauvegarder sur Drive\" pour les enregistrer dans votre dossier Google Drive privé de l'application.",

        _ => key,
    }
}
