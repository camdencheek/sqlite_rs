// TODO: these token names are generated by the lemon parser.
// This is somewhat fragile to redefine them here.
// cbindgen:ignore
pub enum TK {
    SEMI = 1,
    EXPLAIN = 2,
    QUERY = 3,
    PLAN = 4,
    BEGIN = 5,
    TRANSACTION = 6,
    DEFERRED = 7,
    IMMEDIATE = 8,
    EXCLUSIVE = 9,
    COMMIT = 10,
    END = 11,
    ROLLBACK = 12,
    SAVEPOINT = 13,
    RELEASE = 14,
    TO = 15,
    TABLE = 16,
    CREATE = 17,
    IF = 18,
    NOT = 19,
    EXISTS = 20,
    TEMP = 21,
    LP = 22,
    RP = 23,
    AS = 24,
    COMMA = 25,
    WITHOUT = 26,
    ABORT = 27,
    ACTION = 28,
    AFTER = 29,
    ANALYZE = 30,
    ASC = 31,
    ATTACH = 32,
    BEFORE = 33,
    BY = 34,
    CASCADE = 35,
    CAST = 36,
    CONFLICT = 37,
    DATABASE = 38,
    DESC = 39,
    DETACH = 40,
    EACH = 41,
    FAIL = 42,
    OR = 43,
    AND = 44,
    IS = 45,
    MATCH = 46,
    LIKE_KW = 47,
    BETWEEN = 48,
    IN = 49,
    ISNULL = 50,
    NOTNULL = 51,
    NE = 52,
    EQ = 53,
    GT = 54,
    LE = 55,
    LT = 56,
    GE = 57,
    ESCAPE = 58,
    ID = 59,
    COLUMNKW = 60,
    DO = 61,
    FOR = 62,
    IGNORE = 63,
    INITIALLY = 64,
    INSTEAD = 65,
    NO = 66,
    KEY = 67,
    OF = 68,
    OFFSET = 69,
    PRAGMA = 70,
    RAISE = 71,
    RECURSIVE = 72,
    REPLACE = 73,
    RESTRICT = 74,
    ROW = 75,
    ROWS = 76,
    TRIGGER = 77,
    VACUUM = 78,
    VIEW = 79,
    VIRTUAL = 80,
    WITH = 81,
    NULLS = 82,
    FIRST = 83,
    LAST = 84,
    CURRENT = 85,
    FOLLOWING = 86,
    PARTITION = 87,
    PRECEDING = 88,
    RANGE = 89,
    UNBOUNDED = 90,
    EXCLUDE = 91,
    GROUPS = 92,
    OTHERS = 93,
    TIES = 94,
    GENERATED = 95,
    ALWAYS = 96,
    MATERIALIZED = 97,
    REINDEX = 98,
    RENAME = 99,
    CTIME_KW = 100,
    ANY = 101,
    BITAND = 102,
    BITOR = 103,
    LSHIFT = 104,
    RSHIFT = 105,
    PLUS = 106,
    MINUS = 107,
    STAR = 108,
    SLASH = 109,
    REM = 110,
    CONCAT = 111,
    PTR = 112,
    COLLATE = 113,
    BITNOT = 114,
    ON = 115,
    INDEXED = 116,
    STRING = 117,
    JOIN_KW = 118,
    CONSTRAINT = 119,
    DEFAULT = 120,
    NULL = 121,
    PRIMARY = 122,
    UNIQUE = 123,
    CHECK = 124,
    REFERENCES = 125,
    AUTOINCR = 126,
    INSERT = 127,
    DELETE = 128,
    UPDATE = 129,
    SET = 130,
    DEFERRABLE = 131,
    FOREIGN = 132,
    DROP = 133,
    UNION = 134,
    ALL = 135,
    EXCEPT = 136,
    INTERSECT = 137,
    SELECT = 138,
    VALUES = 139,
    DISTINCT = 140,
    DOT = 141,
    FROM = 142,
    JOIN = 143,
    USING = 144,
    ORDER = 145,
    GROUP = 146,
    HAVING = 147,
    LIMIT = 148,
    WHERE = 149,
    RETURNING = 150,
    INTO = 151,
    NOTHING = 152,
    FLOAT = 153,
    BLOB = 154,
    INTEGER = 155,
    VARIABLE = 156,
    CASE = 157,
    WHEN = 158,
    THEN = 159,
    ELSE = 160,
    INDEX = 161,
    ALTER = 162,
    ADD = 163,
    WINDOW = 164,
    OVER = 165,
    FILTER = 166,
    COLUMN = 167,
    AGG_FUNCTION = 168,
    AGG_COLUMN = 169,
    TRUEFALSE = 170,
    ISNOT = 171,
    FUNCTION = 172,
    UMINUS = 173,
    UPLUS = 174,
    TRUTH = 175,
    REGISTER = 176,
    VECTOR = 177,
    SELECT_COLUMN = 178,
    IF_NULL_ROW = 179,
    ASTERISK = 180,
    SPAN = 181,
    ERROR = 182,
    SPACE = 183,
    ILLEGAL = 184,
}
