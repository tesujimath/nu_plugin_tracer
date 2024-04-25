# nu_plugin_tracer

This is a diagnostic wrapper for [Nu plugins](https://www.nushell.sh/book/plugins.html) which dumps the raw plugin stdin and stdout into files in the current directory.

This is probably only useful for people writing plugins in languages other than Rust.

Requires the new plugin API of Nu >= 0.93

The new `local-socket` feature is not currently supported, so plugins must be built with `nu-plugin` dependency with `default-features = false`.

## Usage

Add the plugin using the tracer as a shell interpreter, using the full paths of the plugin tracer and the actual plugin.

```
> plugin add -s ~/vc/sjg/dev.rust/nu_plugin_tracer/target/debug/nu_plugin_tracer ~/vc/tesujimath/nu_plugin_bash_env/nu_plugin_bash_env
> plugin use bash_env
```

The output appears in files named after the plugin, in the current directory.
Note that Nu runs plugins with the current directory set to the plugin installation directory rather than the user's actual current directory. See this [Nushell issue](https://github.com/nushell/nushell/issues/12660).

Then:
```
> cd ~/junk

> echo "export A=123" | bash-env
╭───┬─────╮
│ A │ 123 │
╰───┴─────╯

> cat ~/vc/tesujimath/nu_plugin_bash_env/nu_plugin_bash_env.in.raw
{"Hello":{"protocol":"nu-plugin","version":"0.92.3","features":[{"name":"LocalSocket"}]}}
{"Call":[0,"Signature"]}
"Goodbye"
{"Hello":{"protocol":"nu-plugin","version":"0.92.3","features":[{"name":"LocalSocket"}]}}
{"Call":[0,{"Run":{"name":"bash-env","call":{"head":{"start":131720,"end":131728},"positional":[],"named":[]},"input":{"Value":{"String":{"val":"export A=123","span":{"start":131703,"end":131717}}}}}}]}
"Goodbye"

> cat ~/vc/tesujimath/nu_plugin_bash_env/nu_plugin_bash_env.out.raw
json{"Hello":{"protocol":"nu-plugin","version":"0.92.0","features":[]}}
{"CallResponse":[0,{"Signature":[{"sig":{"name":"bash-env","usage":"get environment variables from Bash format file and/or stdin","extra_usage":"","search_terms":[],"required_positional":[],"optional_positional":[{"name":"path","desc":"path to environment file","shape":"String","var_id":null,"default_value":null}],"rest_positional":null,"named":[{"long":"help","short":"h","arg":null,"required":false,"desc":"Display the help message for this command","var_id":null,"default_value":null}],"input_output_types":[["Nothing","Any"],["String","Any"]],"allow_variants_without_examples":true,"is_filter":true,"creates_scope":false,"allows_unknown_args":false,"category":"Env"},"examples":[]}]}]}
json{"Hello":{"protocol":"nu-plugin","version":"0.92.0","features":[]}}
{"CallResponse":[0,{"PipelineData":{"Value":{"Record":{"val":{"A":{"String":{"val":"123","span":{"start":0,"end":0}}}},"span":{"start":0,"end":0}}}}}]}
```

Note that there is an invisible `\004` character before each occurence of the string `json`.

````
> od -c nu_plugin_bash_env.out.raw | head -1
0000000 004   j   s   o   n   {   "   H   e   l   l   o   "   :   {   "
```
