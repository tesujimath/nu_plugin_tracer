# nu_plugin_tracer

This is a diagnostic wrapper for [Nu plugins](https://www.nushell.sh/book/plugins.html) which dumps the raw plugin stdin and stdout into files in the current directory.

This is probably only useful for people writing plugins in languages other than Rust.

## Usage

The program needs to be installed somewhere on the Nu plugin path, with a name corresponding to the plugin desiring to be wrapped.  It is insufficient for this to be a symlink, because Nu canonicalizes paths, resulting in the tracer program not being able to determine its name as invoked as a plugin.

Then, the tracer plugin can be registered by its full name, and will invoke the wrapped plugin.

The wrapped plugin must reside in the main Nu plugins directory under the Nu config directory.  It is not sufficient for it to be on the Nu plugin path, because the tracer program does not know how to resolve that path.

For example, to wrap `nu_plugin_bash_env`, place a copy of `nu_plugin_tracer` called `nu_plugin_bash_env_tracer` into a directory on the Nu plugin path.

Then:
```
> register nu_plugin_bash_env_tracer
> echo "export A=123" | bash-env
╭───┬─────╮
│ A │ 123 │
╰───┴─────╯

> cat nu_plugin_bash_env.in.raw
{"Hello":{"protocol":"nu-plugin","version":"0.91.0","features":[]}}
{"Call":[0,"Signature"]}
"Goodbye"
{"Hello":{"protocol":"nu-plugin","version":"0.91.0","features":[]}}
{"Call":[0,"Signature"]}
"Goodbye"
{"Hello":{"protocol":"nu-plugin","version":"0.91.0","features":[]}}
{"Call":[0,"Signature"]}
"Goodbye"
{"Hello":{"protocol":"nu-plugin","version":"0.91.0","features":[]}}
{"Call":[0,"Signature"]}
"Goodbye"
{"Hello":{"protocol":"nu-plugin","version":"0.91.0","features":[]}}
{"Call":[0,{"Run":{"name":"bash-env","call":{"head":{"start":118886,"end":118894},"positional":[],"named":[]},"input":{"Value":{"String":{"val":"export A=123","span":{"start":118869,"end":118883}}}},"config":null}}]}
"Goodbye"

> cat nu_plugin_bash_env.out.raw
json{"Hello":{"protocol":"nu-plugin","version":"0.91.0","features":[]}}
{"CallResponse":[0,{"Signature":[{"sig":{"name":"bash-env","usage":"get environment variables from Bash format file and/or stdin","extra_usage":"","search_terms":[],"required_positional":[],"optional_positional":[{"name":"path","desc":"path to environment file","shape":"String","var_id":null,"default_value":null}],"rest_positional":null,"named":[{"long":"help","short":"h","arg":null,"required":false,"desc":"Display the help message for this command","var_id":null,"default_value":null}],"input_output_types":[["Nothing","Any"],["String","Any"]],"allow_variants_without_examples":true,"is_filter":true,"creates_scope":false,"allows_unknown_args":false,"category":"Env"},"examples":[]}]}]}
json{"Hello":{"protocol":"nu-plugin","version":"0.91.0","features":[]}}
{"CallResponse":[0,{"Signature":[{"sig":{"name":"bash-env","usage":"get environment variables from Bash format file and/or stdin","extra_usage":"","search_terms":[],"required_positional":[],"optional_positional":[{"name":"path","desc":"path to environment file","shape":"String","var_id":null,"default_value":null}],"rest_positional":null,"named":[{"long":"help","short":"h","arg":null,"required":false,"desc":"Display the help message for this command","var_id":null,"default_value":null}],"input_output_types":[["Nothing","Any"],["String","Any"]],"allow_variants_without_examples":true,"is_filter":true,"creates_scope":false,"allows_unknown_args":false,"category":"Env"},"examples":[]}]}]}
json{"Hello":{"protocol":"nu-plugin","version":"0.91.0","features":[]}}
{"CallResponse":[0,{"Signature":[{"sig":{"name":"bash-env","usage":"get environment variables from Bash format file and/or stdin","extra_usage":"","search_terms":[],"required_positional":[],"optional_positional":[{"name":"path","desc":"path to environment file","shape":"String","var_id":null,"default_value":null}],"rest_positional":null,"named":[{"long":"help","short":"h","arg":null,"required":false,"desc":"Display the help message for this command","var_id":null,"default_value":null}],"input_output_types":[["Nothing","Any"],["String","Any"]],"allow_variants_without_examples":true,"is_filter":true,"creates_scope":false,"allows_unknown_args":false,"category":"Env"},"examples":[]}]}]}
json{"Hello":{"protocol":"nu-plugin","version":"0.91.0","features":[]}}
{"CallResponse":[0,{"Signature":[{"sig":{"name":"bash-env","usage":"get environment variables from Bash format file and/or stdin","extra_usage":"","search_terms":[],"required_positional":[],"optional_positional":[{"name":"path","desc":"path to environment file","shape":"String","var_id":null,"default_value":null}],"rest_positional":null,"named":[{"long":"help","short":"h","arg":null,"required":false,"desc":"Display the help message for this command","var_id":null,"default_value":null}],"input_output_types":[["Nothing","Any"],["String","Any"]],"allow_variants_without_examples":true,"is_filter":true,"creates_scope":false,"allows_unknown_args":false,"category":"Env"},"examples":[]}]}]}
json{"Hello":{"protocol":"nu-plugin","version":"0.91.0","features":[]}}
{"CallResponse":[0,{"PipelineData":{"Value":{"Record":{"val":{"cols":["A"],"vals":[{"String":{"val":"123","span":{"start":0,"end":0}}}]},"span":{"start":0,"end":0}}}}}]}
```

Note that there is an invisible `\004` character before each occurence of the string `json`.

````
> od -c nu_plugin_bash_env.out.raw | head -1
0000000 004   j   s   o   n   {   "   H   e   l   l   o   "   :   {   "
```
