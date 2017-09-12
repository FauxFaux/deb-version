## deb-version

Here lies a series of ports of the "Debian Version" comparison function to different
languages:

 * `rust`
 
(Okay, just one so far!)

This allows you to work out if `1.0` is newer than `1.0~beta5` (it's not!).

The implemented specification is as of the Debian Policy 4.1.0.0,
section 5.6.12: Version. The (GPL) text is quoted below.

## Specification

The version number of a package. The format is: `[epoch:]upstream_version[-debian_revision]`.

The three components here are:

 * `epoch`: This is a single (generally small) unsigned integer. It may be omitted, 
    in which case zero is assumed. If it is omitted then the `upstream_version` may 
    not contain any colons.

    It is provided to allow mistakes in the version numbers of older versions of a
    package, and also a package's previous version numbering schemes, to be left behind.

 * `upstream_version`: This is the main part of the version number. It is usually
    the version number of the original ("upstream") package from which the .deb
    file has been made, if this is applicable. Usually this will be in the same
    format as that specified by the upstream author(s); however, it may need to
    be reformatted to fit into the package management system's format and comparison scheme.

    The comparison behavior of the package management system with respect to the
    `upstream_version` is described below. The upstream_version portion of the
    version number is mandatory.

    The upstream_version may contain only alphanumerics (`/A-Za-z0-9/`) and the
    characters . + - ~ (full stop, plus, hyphen, tilde) and should start with a digit.
    If there is no `debian_revision` then hyphens are not allowed.

 * `debian_revision`: This part of the version number specifies the version of the Debian
   package based on the upstream version. It may contain only alphanumerics and the
   characters + . ~ (plus, full stop, tilde) and is compared in the same way as the
   `upstream_version` is.

   It is optional; if it isn't present then the upstream_version may not contain a hyphen.
   This format represents the case where a piece of software was written specifically
   to be a Debian package, where the Debian package source must always be identical
   to the pristine source and therefore no revision indication is required.

It is conventional to restart the `debian_revision` at 1 each time the `upstream_version`
 is increased.

The package management system will break the version number apart at the last hyphen in
the string (if there is one) to determine the `upstream_version` and `debian_revision`.
The absence of a `debian_revision` is equivalent to a `debian_revision` of 0.

When comparing two version numbers, first the epoch of each are compared, then the
`upstream_version` if `epoch` is equal, and then `debian_revision` if `upstream_version`
is also equal. `epoch` is compared numerically. The `upstream_version` and `debian_revision`
parts are compared by the package management system using the following algorithm:

The strings are compared from left to right.

First the initial part of each string consisting entirely of non-digit characters
is determined. These two parts (one of which may be empty) are compared lexically.
If a difference is found it is returned. The lexical comparison is a comparison of ASCII
values modified so that all the letters sort earlier than all the non-letters and so that
a tilde sorts before anything, even the end of a part. For example, the following parts are
in sorted order from earliest to latest: `~~`, `~~a`, `~`, the empty part, `a`.

Then the initial part of the remainder of each string which consists entirely of digit
characters is determined. The numerical values of these two parts are compared, and any
difference found is returned as the result of the comparison. For these purposes an empty
string (which can only occur at the end of one or both version strings being compared)
counts as zero.

These two steps (comparing and removing initial non-digit strings and initial digit strings)
are repeated until a difference is found or both strings are exhausted.

## License

The code here is available under the MIT license. The above specification, and the tests
against `libapt`, require components that are GPL, but the actual code is all available
under the MIT license: you are free to use it anywhere with no restrictions.
