//! Abstract syntax tree.

use url::Url;

/// A [document][].
///
/// This represents an entire reStructuredText document and forms the root of the tree.
///
/// [document]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#document
pub struct Document(Body);

/// Characters that may be used as adornments.
///
/// The following are all valid adornment characters:
///
/// ```text
/// ! " # $ % & ' ( ) * + , - . / : ; < = > ? @ [ \ ] ^ _ ` { | } ~
/// ```
///
/// Some characters are more suitable than others. The following are recommended:
///
/// ```text
/// = - ` : . ' " ~ ^ _ * + #
/// ```
pub const ADORNMENT_CHARS: &[char] = &[
    '!', '"', '#', '$', '%', '&', '\'', '(',
    ')', '*', '+', ',', '-', '.', '/', ':',
    ';', '<', '=', '>', '?', '@', '[', '\\',
    ']', '^', '_', '`', '{', '|', '}', '~',
];

/// A [section][].
///
/// Sections are identified through their titles, which are marked up with adornment: "underlines"
/// below the title text, or underlines and matching "overlines" above the title. An
/// underline/overline is a single repeated punctuation character that begins in column 1 and forms
/// a line extending at least as far as the right edge of the title text.
///
/// Adornments must use characters from the set of [adornment
/// characters](constant.ADORNMENT_CHARS.html);
///
/// ```rst
/// ===============
///  Section Title
/// ===============
///
/// ---------------
///  Section Title
/// ---------------
///
/// Section Title
/// =============
///
/// Section Title
/// -------------
/// ```
///
/// Sections may be segmented using [transitions][].
///
/// > Instead of subheads, extra space or a type ornament between paragraphs may be used to mark
/// > text divisions or to signal changes in subject or emphasis.
/// >
/// > -- (The Chicago Manual of Style, 14th edition, section 1.80)
///
/// A transition should not begin or end a section or document, nor should two transitions be
/// immediately adjacent.
///
/// A transition is marked by a line of 4 or more of an
/// [adornment character](constant.ADORNMENT_CHARS.html).
///
/// ```rst
/// Para.
///
/// ----------
///
/// Para.
/// ```
///
/// [section]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#sections
/// [transitions]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#transitions
pub struct Section {
    title: String,
    children: Vec<SectionChildren>,
}

/// Children of a section.
enum SectionChildren {
    Body(BodyBlock),
    Transition,
    Section(Section),
}

/// A block that can be embedded within the body of another element.
pub enum BodyBlock {
    Paragraph(Paragraph),
    List(List),
    DefinitionList(DefinitionList),
    FieldList(FieldList),
    OptionList(OptionList),
    LiteralBlock(LiteralBlock),
    LineBlock(LineBlock),
    BlockQuote(BlockQuote),
    DocTest(DocTest),
    Table(Table),
    Footnote(Footnote),
    Citation(Citation),
    Target(Target),
    Directive(Directive),
    Substitution(Substitution),
    Comment(Comment),
}

/// A sequence of [`BodyBlock`](enum.BodyBlock.html)s.
pub struct Body(Vec<BodyBlock>);

/// A [paragraph][].
///
/// Paragraphs consist of blocks of left-aligned text with no markup indicating any other body
/// element. Blank lines separate paragraphs from each other and from other body elements.
/// Paragraphs may contain [inline markup](struct.Text.html).
///
/// [paragraph]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#paragraphs
pub struct Paragraph(Text);

/// A list; [bulleted][] or [enumerated][];
///
/// A text block which begins with a "*", "+", "-", "•", "‣", or "⁃", followed by whitespace, is a
/// bullet list item (a.k.a. "unordered" list item).
///
/// ```rst
/// - This is the first bullet list item.  The blank line above the
///   first list item is required; blank lines between list items
///   (such as below this paragraph) are optional.
///
/// - This is the first paragraph in the second item in the list.
///
///   This is the second paragraph in the second item in the list.
///   The blank line above this paragraph is required.  The left edge
///   of this paragraph lines up with the paragraph above, both
///   indented relative to the bullet.
///
///   - This is a sublist.  The bullet lines up with the left edge of
///     the text blocks above.  A sublist is a new list so requires a
///     blank line above and below.
///
/// - This is the third item of the main list.
///
/// This paragraph is not part of the list.
/// ```
///
/// Enumerated lists (a.k.a. "ordered" lists) are similar to bullet lists, but use enumerators
/// instead of bullets. An enumerator consists of an enumeration sequence member and formatting,
/// followed by whitespace.
///
/// ```rst
/// 1. Item 1 initial text.
///
///    a) Item 1a.
///    b) Item 1b.
///
/// 2. a) Item 2a.
///    b) Item 2b.
/// ```
///
/// [bulleted]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#bullet-lists
/// [enumerated]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#enumerated-lists
pub struct List {
    marker: ListMarker,
    elements: Vec<Body>,
}

/// The kind of marker used to identify elements of the list.
///
/// For enumerated lists, the starting index is also provided.
enum ListMarker {
    /// A standard bulleted list.
    Bullet,
    /// A list enumerated with arabic decimals.
    Arabic(u64),
    /// A list enumerated with uppercase latin letters.
    LatinUppercase(u64),
    /// A list enumerated with lowercase latin letters.
    LatinLowercase(u64),
    /// A list enumerate with uppercase roman numerals.
    RomanUppercase(u64),
    /// A list enumerate with lowercase roman numerals.
    RomanLowercase(u64),
}

/// A [definition list][].
///
/// Each definition list item contains a term, optional classifiers, and a definition. A term is a
/// simple one-line word or phrase. Optional classifiers may follow the term on the same line, each
/// after an inline " : " (space, colon, space). A definition is a block indented relative to the
/// term, and may contain multiple paragraphs and other body elements.
///
/// ```rst
/// term 1
///     Definition 1.
///
/// term 2
///     Definition 2, paragraph 1.
///
///     Definition 2, paragraph 2.
///
/// term 3 : classifier
///     Definition 3.
///
/// term 4 : classifier one : classifier two
///     Definition 4.
/// ```
///
/// [definition list]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#definition-lists.
pub struct DefinitionList(Vec<Definition>);

/// A single definition within a [`DefinitionList`](struct.DefinitionList.html).
pub struct Definition {
    term: Text,
    classifiers: Vec<Text>,
    definition: Body,
}

/// A [field list][].
///
/// Field lists are mappings from field names to field bodies, modeled on [RFC822][] headers. A
/// field name may consist of any characters, but colons (":") inside of field names must be
/// backslash-escaped when followed by whitespace.
///
/// ```rst
/// :Date: 2001-08-16
/// :Version: 1
/// :Authors: - Me
///           - Myself
///           - I
/// :Indentation: Since the field marker may be quite long, the second
///    and subsequent lines of the field body do not have to line up
///    with the first line, but they must be indented relative to the
///    field name marker, and they must line up with each other.
/// :Parameter i: integer
/// ```
///
/// [field list]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#field-lists
/// [rfc822]: http://www.rfc-editor.org/rfc/rfc822.txt
pub struct FieldList(Vec<Field>);

/// An element of a [`FieldList`](struct.FieldList.html).
pub struct Field {
    marker: Text,
    body: Body,
}

/// An [option list][].
///
/// Option lists are two-column lists of command-line options and descriptions, documenting a
/// program's options. For example:
///
/// ```rst
/// -a         Output all.
/// -b         Output both (this description is
///            quite long).
/// -c arg     Output just arg.
/// --long     Output all day long.
///
/// -p         This option has two paragraphs in the description.
///            This is the first.
///
///            This is the second.  Blank lines may be omitted between
///            options (as above) or left in (as here and below).
///
/// --very-long-option  A VMS-style option.  Note the adjustment for
///                     the required two spaces.
///
/// --an-even-longer-option
///            The description can also start on the next line.
///
/// -2, --two  This option has two variants.
///
/// -f FILE, --file=FILE  These two options are synonyms; both have
///                       arguments.
///
/// /V         A VMS/DOS-style option.
/// ```
///
/// There are several types of options recognized by reStructuredText:
///
///  * Short POSIX options consist of one dash and an option letter.
///  * Long POSIX options consist of two dashes and an option word; some systems use a single
///    dash.
///  * Old GNU-style "plus" options consist of one plus and an option letter ("plus" options
///    are deprecated now, their use discouraged).
///  * DOS/VMS options consist of a slash and an option letter or word.
///
/// [option list]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#option-lists
pub struct OptionList(Vec<OptionItem>);

/// An item within an [`OptionList`](struct.OptionList.html).
pub struct OptionItem {
    options: Vec<(String, Option<String>)>,
    description: Text,
}

/// A [literal block][].
///
/// A paragraph consisting of two colons ("::") signifies that the following text block(s)
/// comprise a literal block. The literal block must either be indented or quoted (see below).
/// No markup processing is done within a literal block.
///
/// ```rst
/// This is a typical paragraph.  An indented literal block follows.
///
/// ::
///
///     for a in [5,4,3,2,1]:   # this is program code, shown as-is
///         print a
///     print "it's..."
///     # a literal block continues until the indentation ends
///
/// This text has returned to the indentation of the first paragraph,
/// is outside of the literal block, and is therefore treated as an
/// ordinary paragraph.
/// ```
///
/// The following demonstrate the minimised form of the literal block and are all
/// equivalent:
///
/// ```rst
/// Paragraph:
///
/// ::
///
///     Literal block
/// ```
///
/// ```rst
/// Paragraph: ::
///
///     Literal block
/// ```
///
/// ```rst
/// Paragraph::
///
///     Literal block
/// ```
///
/// Literal blocks may also be quoted using [adornmemt
/// characters](constant.ADORNMENT_CHARS.html).
///
/// ```rst
/// John Doe wrote::
///
/// >> Great idea!
/// >
/// > Why didn't I think of that?
///
/// You just did!  ;-)
/// ```
///
/// [literal block]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#literal-blocks
pub struct LiteralBlock(String);

/// A [line block][].
///
/// Line blocks are useful for address blocks, verse (poetry, song lyrics), and unadorned
/// lists, where the structure of lines is significant. Line blocks are groups of lines
/// beginning with vertical bar ("|") prefixes. Each vertical bar prefix indicates a new line,
/// so line breaks are preserved. Initial indents are also significant, resulting in a nested
/// structure. Inline markup is supported. Continuation lines are wrapped portions of long
/// lines; they begin with a space in place of the vertical bar. The left edge of a
/// continuation line must be indented, but need not be aligned with the left edge of the text
/// above it. A line block ends with a blank line.
///
/// ```rst
/// Take it away, Eric the Orchestra Leader!
///
///    | A one, two, a one two three four
///    |
///    | Half a bee, philosophically,
///    |     must, *ipso facto*, half not be.
///    | But half the bee has got to be,
///    |     *vis a vis* its entity.  D'you see?
///    |
///    | But can a bee be said to be
///    |     or not to be an entire bee,
///    |         when half the bee is not a bee,
///    |             due to some ancient injury?
///    |
///    | Singing...
/// ```
///
/// [line block]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#line-blocks
pub struct LineBlock(Vec<Line>);

/// A line within a [`LineBlock`](struct.LineBlock.html).
pub struct Line {
    content: Text,
    children: Vec<Line>,
}

/// A [block quote][].
///
/// A text block that is indented relative to the preceding text, without preceding markup
/// indicating it to be a literal block or other content, is a block quote.
///
/// ```rst
/// This is an ordinary paragraph, introducing a block quote.
///
///     "It is my business to know things.  That is my trade."
///
///     -- Sherlock Holmes
/// ```
///
/// [block quote]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#block-quotes
pub struct BlockQuote {
    quote: Body,
    attribution: Option<Text>,
}

/// A [doctest block][].
///
/// Doctest blocks are interactive Python sessions cut-and-pasted into docstrings. They are
/// meant to illustrate usage by example, and provide an elegant and powerful testing
/// environment via the [doctest module][] in the Python standard library.
///
/// ```rst
/// This is an ordinary paragraph.
///
/// >>> print 'this is a Doctest block'
/// this is a Doctest block
///
/// The following is a literal block::
///
///     >>> This is not recognized as a doctest block by
///     reStructuredText.  It *will* be recognized by the doctest
///     module, though!
/// ```
///
/// [doctest block]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#doctest-blocks
/// [doctest module]: http://www.python.org/doc/current/lib/module-doctest.html
pub struct DocTest(String);

/// A [table][].
///
/// Grid tables provide a complete table representation via grid-like "ASCII art". Grid tables
/// allow arbitrary cell contents (body elements), and both row and column spans.
///
/// ```rst
/// +------------------------+------------+----------+----------+
/// | Header row, column 1   | Header 2   | Header 3 | Header 4 |
/// | (header rows optional) |            |          |          |
/// +========================+============+==========+==========+
/// | body row 1, column 1   | column 2   | column 3 | column 4 |
/// +------------------------+------------+----------+----------+
/// | body row 2             | Cells may span columns.          |
/// +------------------------+------------+---------------------+
/// | body row 3             | Cells may  | - Table cells       |
/// +------------------------+ span rows. | - contain           |
/// | body row 4             |            | - body elements.    |
/// +------------------------+------------+---------------------+
/// ```
///
/// Simple tables provide a compact and easy to type but limited row-oriented table
/// representation for simple data sets. Cell contents are typically single paragraphs,
/// although arbitrary body elements may be represented in most cells.
///
/// ```rst
/// =====  =====  ======
///    Inputs     Output
/// ------------  ------
///   A      B    A or B
/// =====  =====  ======
/// False  False  False
/// True   False  True
/// False  True   True
/// True   True   True
/// =====  =====  ======
/// ```
///
/// [table]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#tables
pub struct Table {
    header: Vec<Row>,
    body: Vec<Row>,
}

/// Rows within a [`Table`](struct.Table.html).
pub struct Row(Vec<Cell>);

/// A cell within a [`Table`](struct.Table.html).
pub struct Cell {
    column_span: u64,
    row_span: u64,
    content: Text,
}

/// A [footnote][].
///
/// Each footnote consists of an explicit markup start (".. "), a left square bracket, the
/// footnote label, a right square bracket, and whitespace, followed by indented body elements.
///
/// ```rst
/// If [#note]_ is the first footnote reference, it will show up as
/// "[1]".  We can refer to it again as [#note]_ and again see
/// "[1]".  We can also refer to it as note_ (an ordinary internal
/// hyperlink reference).
///
/// .. [#note] This is the footnote labeled "note".
/// ```
///
/// ```rst
/// [#]_ is a reference to footnote 1, and [#]_ is a reference to
/// footnote 2.
///
/// .. [#] This is footnote 1.
/// .. [#] This is footnote 2.
/// .. [#] This is footnote 3.
///
/// [#]_ is a reference to footnote 3.
/// ```
///
/// ```rst
/// Here is a symbolic footnote reference: [*]_.
///
/// .. [*] This is the footnote.
/// ```
///
/// [footnote]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#footnotes
pub struct Footnote {
    identifier: FootnoteIdentifier,
    body: Body,
}

/// An identifier of a particular [`Footnote`](struct.Footnote.html).
pub enum FootnoteIdentifier {
    AutoNumbered,
    Numbered(u64),
    Labelled(String),
}

/// A [citation][].
///
/// Citations are identical to footnotes except that they use only non-numeric labels such as
/// `[note]` or `[GVR2001]`. Citation labels are simple [reference names][] (case-insensitive single
/// words consisting of alphanumerics plus internal hyphens, underscores, and periods; no
/// whitespace).
///
/// ```rst
/// Here is a citation reference: [CIT2002]_.
///
/// .. [CIT2002] This is the citation.  It's just like a footnote,
///    except the label is textual.
/// ```
///
/// [citation]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#citations
/// [reference names]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#reference-names
pub struct Citation {
    name: String,
    body: Body,
}

/// A [hyperlink target][].
///
/// Hyperlink targets identify a location within or outside of a document, which may be linked to by [hyperlink references](struct.HyperlinkReference.html).
///
/// ```rst
/// .. _hyperlink-name: link-block
///
/// .. __: anonymous-hyperlink-target-link-block
///
/// __ anonymous-hyperlink-target-link-block
///
/// .. _Python DOC-SIG mailing list archive:
/// .. _archive:
/// .. _Doc-SIG: http://mail.python.org/pipermail/doc-sig/
/// ```
///
/// [hyperlink target]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#hyperlink-targets
pub struct Target;

/// The content referred to by a [`Target`](struct.Target.html).
pub enum HyperlinkContent {
    Empty,
    URI(Url),
    Email(String),
    Reference(String),
}

/// A [directive][].
///
/// Directives are an extension mechanism for reStructuredText, a way of adding support for new
/// constructs without adding new primary syntax (directives may support additional syntax
/// locally).
///
/// Directives are indicated by an explicit markup start (".. ") followed by the directive
/// type, two colons, and whitespace (together called the "directive marker"). Directive types
/// are case-insensitive single words (alphanumerics plus isolated internal hyphens,
/// underscores, plus signs, colons, and periods; no whitespace).
///
/// ```rst
/// .. image:: mylogo.jpeg
///
/// .. figure:: larch.png
///
///    The larch.
///
/// .. note:: This is a paragraph
///
///    - Here is a bullet list.
/// ```
///
/// [directive]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#directives
pub struct Directive {
    marker: String,
    fields: FieldList,
    content: DirectiveContent,
}

/// The content of a [`Directive`](struct.Directive.html).
///
/// As some directives may want their content to be pre-processed as part of the
/// document, the contents may need to be processed to become part of the syntax tree.
pub enum DirectiveContent {
    Literal(String),
    Parsed(Body),
}

/// A [substitution definition][].
///
/// Substitution definitions are indicated by an explicit markup start (".. ") followed by a
/// vertical bar, the substitution text, another vertical bar, whitespace, and the definition
/// block. Substitution text may not begin or end with whitespace.
///
/// ```rst
/// The |biohazard| symbol must be used on containers used to
/// dispose of medical waste.
///
/// .. |biohazard| image:: biohazard.png
/// ```
///
/// [substitution definition]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#substitution-definitions
pub struct Substitution {
    text: String,
    directive: Directive,
}

/// A [comment][].
///
/// Arbitrary indented text may follow the explicit markup start and will be processed as a
/// comment element. No further processing is done on the comment block text; a comment
/// contains a single "text blob".
///
/// ```rst
/// .. This is a comment
/// ..
///    _so: is this!
/// ..
///    [and] this!
/// ..
///    this:: too!
/// ..
///    |even| this:: !
/// ```
///
/// [comment]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#comments
pub struct Comment(String);

/// An [inline][] item.
///
/// [inline]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#inline-markup
pub enum Inline {
    Emphasis(Emphasis),
    Strong(Strong),
    Interpreted(Interpreted),
    Literal(Literal),
    HyperlinkReference(HyperlinkReference),
    Target(InlineInternalTarget),
    StandaloneHyperlink(StandaloneHyperlink),
    Unit(Unit),
    Word(String),
    Character(char),
    Whitespace,
}

/// A sequence of [`Inline`](enum.Inline.html) items.
pub struct Text(Vec<Inline>);

/// Text [emphasis][].
///
/// [emphasis]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#emphasis
pub struct Emphasis;

/// [Strong][] text emphasis.
///
/// [strong]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#strong-emphasis
pub struct Strong;

/// [Interpreted][] text.
///
/// [interpreted]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#interpreted-text
pub struct Interpreted;

/// An inline [literal][].
///
/// [literal]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#inline-literals
pub struct Literal;

/// A [hyperlink reference][].
///
/// [hyperlink reference]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#hyperlink-references
pub struct HyperlinkReference;

/// An [inline internal target][].
///
/// [inline internal target]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#inline-internal-targets
pub struct InlineInternalTarget;

/// A [footnote reference][].
///
/// [footnote reference]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#footnote-references
pub struct FootnoteReference;

/// A [substitution reference][].
///
/// [substitution reference]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#substitution-references
pub struct SubstitutionReference;

/// A [standalone hyperlink][].
///
/// [standalone hyperlink]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#standalone-hyperlinks
pub struct StandaloneHyperlink;

/// A [unit][] of measure;
///
/// [unit]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html#units
pub enum Unit {
    Em(f64),
    Ex(f64),
    Millimeter(f64),
    Centimeter(f64),
    Inch(f64),
    Pixel(f64),
    Point(f64),
    Pica(f64),
    Percent(f64),
}
