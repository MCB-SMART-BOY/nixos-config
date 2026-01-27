{ ... }:

let
  editor = "hx";
in
{
  programs.git = {
    enable = true;
    lfs.enable = true;
    settings = {
      core = {
        editor = editor;
        pager = "delta";
      };
      interactive.diffFilter = "delta --color-only";
      delta = {
        navigate = true;
        "side-by-side" = true;
      };
    };
  };
}
