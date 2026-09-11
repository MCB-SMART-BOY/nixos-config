{ lib }:

{
  validUserName = name: builtins.match"[a-z_][a-z0-9_-]*" name != null;

  resolvePkg = path: pkgSet:
    let eval = if lib.hasAttrByPath path pkgSet
      then builtins.tryEval (lib.getAttrFromPath path pkgSet)
      else {
        success = false;
        value = null;
      };
    in
      if eval.success
        then eval.value
        else null;
}
