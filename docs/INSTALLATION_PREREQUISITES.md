# Capability prerequisites — generated release-note fragment

The native core, project open, plotting and native exports do not require Python. The following
optional capabilities use one session-resolved Python 3.10+ subprocess:

- **Python equations** — requires numpy; optional scipy (owner: `SB-MLA`).
- **DLIS import** — requires dlisio (owner: `SB-DIO`).
- **Spreadsheet plate extraction** — requires openpyxl, Pillow (owner: `SB-DIO`).
- **Workbook export** — requires xlsxwriter (owner: `SB-DIO`).
- **Document export** — requires python-docx (owner: `SB-DIO`).
- **Deck export** — requires python-pptx, matplotlib (owner: `SB-DIO`).

The supported offline route is the separately signed, versioned SandiBumi-qualified Python pack,
silently deployed per machine by IT. It configures `SANDIBUMI_PYTHON` to its application-local
interpreter. The release gate blocks public network access and accepts zero observed requests.
Exact package versions are supplied only by that release's qualification lock.
