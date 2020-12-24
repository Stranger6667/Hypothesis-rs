import sys
import tempfile
import urllib
import subprocess
import urllib.request
import zipfile


CATEGORY_ABBREVIATIONS = {
    "Close_Punctuation": "Pe",
    "Connector_Punctuation": "Pc",
    "Control": "Cc",
    "Currency_Symbol": "Sc",
    "Dash_Punctuation": "Pd",
    "Decimal_Number": "Nd",
    "Enclosing_Mark": "Me",
    "Final_Punctuation": "Pf",
    "Format": "Cf",
    "Initial_Punctuation": "Pi",
    "Letter_Number": "Nl",
    "Line_Separator": "Zl",
    "Lowercase_Letter": "Ll",
    "Math_Symbol": "Sm",
    "Modifier_Letter": "Lm",
    "Modifier_Symbol": "Sk",
    "Nonspacing_Mark": "Mn",
    "Open_Punctuation": "Ps",
    "Other_Letter": "Lo",
    "Other_Number": "No",
    "Other_Punctuation": "Po",
    "Other_Symbol": "So",
    "Paragraph_Separator": "Zp",
    "Private_Use": "Co",
    "Space_Separator": "Zs",
    "Spacing_Mark": "Mc",
    "Surrogate": "Cs",
    "Titlecase_Letter": "Lt",
    "Unassigned": "Cn",
    "Uppercase_Letter": "Lu",
}


def main() -> None:
    unicode_version = sys.argv[1]
    response, _ = urllib.request.urlretrieve(f"https://www.unicode.org/Public/zipped/{unicode_version}/UCD.zip")
    directory = tempfile.mkdtemp()
    with zipfile.ZipFile(response, "r") as compressed:
        compressed.extractall(directory)
    version = unicode_version.replace(".", "_")
    filename = f"src/tables/v{version}.rs"
    subprocess.run(
        [
            f"ucd-generate general-category {directory} "
            f"--exclude=Separator,Symbol,Cased_Letter,Letter,Mark,Number,Other,Punctuation > {filename}",
        ],
        shell=True,
        check=True,
    )
    with open(filename, "r") as fd:
        content = fd.read()
        for category, abbreviation in CATEGORY_ABBREVIATIONS.items():
            content = content.replace(category, abbreviation)
    with open(filename, "w") as fd:
        fd.write(content)


if __name__ == "__main__":
    main()
