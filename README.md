# idris-elba-interface (TBD)

A wrapper for elba that exposes cli api consistent with
idris compiler. It's used to easily integrate elba into
existing editor plugins.

## Usage

1. Clone this repo.
2. Install by `"cargo install --path . [--force]"`
3. Modify idris-compiler-path in your editor setting
to `idris-elba-interface`.

## Manifest Watch

When a `--ide-mode` or `--ide-mode-socket` flag is passed
in, the wrapper will watch file changes of `elba.toml` and
`elba.lock`. Once a change fired, the wrapper will reload 
elba process in background automatically.

## Limitations

Currently only a few cli commands are implemented by the 
wrapper:

- `(no args)` (repl)
- `--build`
- `--ide-mode`
- `--ide-mode-socket`

The plugin of vscode has some strange behaviors after the
reload caused by manifest change. Seems that it's because
the plugin tries to cache previous evaluations. Further tests
on other editors is necessary.

## Todos

- [ ] kill elba child process on wrapper exit by ctrl-c.
- [ ] implement a mechanism that ensures a user have
elba with proper api version.  
- [ ] decide wrapper's name.
- [ ] implement `--check` if we find any plugin relying 
on it.
