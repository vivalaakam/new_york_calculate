{ pkgs, lib, ... }:

{
  dotenv.enable = true;
  # https://devenv.sh/basics/
  env.GREET = "devenv";

  # rust env
  #  env.RUSTC_WRAPPER = "${pkgs.sccache}/bin/sccache";
  env.CARGO_BUILD_RUSTFLAGS = "-C link-arg=-fuse-ld=${pkgs.lld_13}/bin/ld64.lld -C linker=${pkgs.clang_13}/bin/clang";

  packages = (
    with pkgs; [
      git
      cargo-make
      cargo-tarpaulin
    ] ++ lib.optionals stdenv.isDarwin
      (with darwin.apple_sdk; [
        frameworks.SystemConfiguration
        frameworks.Security
        frameworks.CoreFoundation
      ])
  );

  # https://devenv.sh/scripts/
  scripts.hello.exec = "echo hello from $GREET";

  enterShell = ''
    hello
    git --version
  '';

  # https://devenv.sh/languages/
  languages.nix.enable = true;

  languages.rust = {
    enable = true;
    channel = "stable";
    toolchain.rust-analyzer = pkgs.rust-analyzer;
  };

  pre-commit.hooks = {
    # Docs: https://devenv.sh/pre-commit-hooks/
    # available pre-configured hooks: https://devenv.sh/reference/options/#pre-commithooks
    # adding hooks which are not included: https://github.com/cachix/pre-commit-hooks.nix/issues/31
    # nixpkgs-fmt.enable = true; # nix formatting
    nil.enable = true; # nix check

    clippy.enable = true; # rust linter
    rustfmt.enable = true; # rust formatting
    #cargo-check.enable = true; ← alternative to clippy
  };

  #   languages.rust.channel = "stable";

  # https://devenv.sh/pre-commit-hooks/
  # pre-commit.hooks.shellcheck.enable = true;

  # https://devenv.sh/processes/
  # processes.ping.exec = "ping example.com";

  # See full reference at https://devenv.sh/reference/options/
}
