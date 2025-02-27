# Starstruck

Starstruck is a shell prompt generator.
It uses the [Osyris](https://github.com/mortie/osyris) programming language
for configuration.

![Screenshot](screenshot.png)

## Shell integration

### Bash

Add the following line to your `~/.bashrc` file:

```
PS1='$(starstruck --bash -e $?)'
```

### ZSH

Add the following line to your `~/.zshrc` file:

```
PS1='$(starstruck --zsh -e $?)'
```
