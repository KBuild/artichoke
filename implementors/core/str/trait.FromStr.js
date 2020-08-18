(function() {var implementors = {};
implementors["chrono"] = [{"text":"impl FromStr for NaiveDate","synthetic":false,"types":[]},{"text":"impl FromStr for NaiveDateTime","synthetic":false,"types":[]},{"text":"impl FromStr for NaiveTime","synthetic":false,"types":[]},{"text":"impl FromStr for DateTime&lt;Utc&gt;","synthetic":false,"types":[]},{"text":"impl FromStr for DateTime&lt;Local&gt;","synthetic":false,"types":[]},{"text":"impl FromStr for DateTime&lt;FixedOffset&gt;","synthetic":false,"types":[]},{"text":"impl FromStr for Weekday","synthetic":false,"types":[]},{"text":"impl FromStr for Month","synthetic":false,"types":[]}];
implementors["clap"] = [{"text":"impl FromStr for AppSettings","synthetic":false,"types":[]},{"text":"impl FromStr for ArgSettings","synthetic":false,"types":[]},{"text":"impl FromStr for Shell","synthetic":false,"types":[]}];
implementors["log"] = [{"text":"impl FromStr for Level","synthetic":false,"types":[]},{"text":"impl FromStr for LevelFilter","synthetic":false,"types":[]}];
implementors["nix"] = [{"text":"impl FromStr for Signal","synthetic":false,"types":[]}];
implementors["proc_macro2"] = [{"text":"impl FromStr for TokenStream","synthetic":false,"types":[]}];
implementors["regex"] = [{"text":"impl FromStr for Regex","synthetic":false,"types":[]},{"text":"impl FromStr for Regex","synthetic":false,"types":[]}];
implementors["spinoso_symbol"] = [{"text":"impl FromStr for IdentifierType","synthetic":false,"types":[]}];
implementors["termcolor"] = [{"text":"impl FromStr for Color","synthetic":false,"types":[]}];
implementors["uuid"] = [{"text":"impl FromStr for Uuid","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()