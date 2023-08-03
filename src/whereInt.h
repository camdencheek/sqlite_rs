/*
** 2013-11-12
**
** The author disclaims copyright to this source code.  In place of
** a legal notice, here is a blessing:
**
**    May you do good and not evil.
**    May you find forgiveness for yourself and forgive others.
**    May you share freely, never taking more than you give.
**
*************************************************************************
**
** This file contains structure and macro definitions for the query
** planner logic in "where.c".  These definitions are broken out into
** a separate source file for easier editing.
*/
#ifndef SQLITE_WHEREINT_H
#define SQLITE_WHEREINT_H


# define WHERE_LOOP_XFER_SZ offsetof(WhereLoop,nLSlot)

/* Allowed values for WhereLoopBuider.bldFlags */
#define SQLITE_BLDF1_INDEXED  0x0001   /* An index is used */
#define SQLITE_BLDF1_UNIQUE   0x0002   /* All keys of a UNIQUE index used */

#define SQLITE_BLDF2_2NDPASS  0x0004   /* Second builder pass needed */

/* The WhereLoopBuilder.iPlanLimit is used to limit the number of
** index+constraint combinations the query planner will consider for a
** particular query.  If this parameter is unlimited, then certain
** pathological queries can spend excess time in the sqlite3WhereBegin()
** routine.  The limit is high enough that is should not impact real-world
** queries.
**
** SQLITE_QUERY_PLANNER_LIMIT is the baseline limit.  The limit is
** increased by SQLITE_QUERY_PLANNER_LIMIT_INCR before each term of the FROM
** clause is processed, so that every table in a join is guaranteed to be
** able to propose a some index+constraint combinations even if the initial
** baseline limit was exhausted by prior tables of the join.
*/
#ifndef SQLITE_QUERY_PLANNER_LIMIT
# define SQLITE_QUERY_PLANNER_LIMIT 20000
#endif
#ifndef SQLITE_QUERY_PLANNER_LIMIT_INCR
# define SQLITE_QUERY_PLANNER_LIMIT_INCR 1000
#endif

/*
** Private interfaces - callable only by other where.c routines.
**
** where.c:
*/
Bitmask sqlite3WhereGetMask(WhereMaskSet*,int);
#ifdef WHERETRACE_ENABLED
void sqlite3WhereClausePrint(WhereClause *pWC);
void sqlite3WhereTermPrint(WhereTerm *pTerm, int iTerm);
void sqlite3WhereLoopPrint(WhereLoop *p, WhereClause *pWC);
#endif
WhereTerm *sqlite3WhereFindTerm(
  WhereClause *pWC,     /* The WHERE clause to be searched */
  int iCur,             /* Cursor number of LHS */
  int iColumn,          /* Column number of LHS */
  Bitmask notReady,     /* RHS must not overlap with this mask */
  u32 op,               /* Mask of WO_xx values describing operator */
  Index *pIdx           /* Must be compatible with this index, if not NULL */
);
void *sqlite3WhereMalloc(WhereInfo *pWInfo, u64 nByte);
void *sqlite3WhereRealloc(WhereInfo *pWInfo, void *pOld, u64 nByte);

/* wherecode.c: */
#ifndef SQLITE_OMIT_EXPLAIN
int sqlite3WhereExplainOneScan(
  Parse *pParse,                  /* Parse context */
  SrcList *pTabList,              /* Table list this loop refers to */
  WhereLevel *pLevel,             /* Scan to write OP_Explain opcode for */
  u16 wctrlFlags                  /* Flags passed to sqlite3WhereBegin() */
);
int sqlite3WhereExplainBloomFilter(
  const Parse *pParse,            /* Parse context */
  const WhereInfo *pWInfo,        /* WHERE clause */
  const WhereLevel *pLevel        /* Bloom filter on this level */
);
#else
# define sqlite3WhereExplainOneScan(u,v,w,x) 0
# define sqlite3WhereExplainBloomFilter(u,v,w) 0
#endif /* SQLITE_OMIT_EXPLAIN */
#ifdef SQLITE_ENABLE_STMT_SCANSTATUS
void sqlite3WhereAddScanStatus(
  Vdbe *v,                        /* Vdbe to add scanstatus entry to */
  SrcList *pSrclist,              /* FROM clause pLvl reads data from */
  WhereLevel *pLvl,               /* Level to add scanstatus() entry for */
  int addrExplain                 /* Address of OP_Explain (or 0) */
);
#else
# define sqlite3WhereAddScanStatus(a, b, c, d) ((void)d)
#endif
Bitmask sqlite3WhereCodeOneLoopStart(
  Parse *pParse,       /* Parsing context */
  Vdbe *v,             /* Prepared statement under construction */
  WhereInfo *pWInfo,   /* Complete information about the WHERE clause */
  int iLevel,          /* Which level of pWInfo->a[] should be coded */
  WhereLevel *pLevel,  /* The current level pointer */
  Bitmask notReady     /* Which tables are currently available */
);
SQLITE_NOINLINE void sqlite3WhereRightJoinLoop(
  WhereInfo *pWInfo,
  int iLevel,
  WhereLevel *pLevel
);

/* whereexpr.c: */
void sqlite3WhereClauseInit(WhereClause*,WhereInfo*);
void sqlite3WhereClauseClear(WhereClause*);
void sqlite3WhereSplit(WhereClause*,Expr*,u8);
void sqlite3WhereAddLimit(WhereClause*, Select*);
Bitmask sqlite3WhereExprUsage(WhereMaskSet*, Expr*);
Bitmask sqlite3WhereExprUsageNN(WhereMaskSet*, Expr*);
Bitmask sqlite3WhereExprListUsage(WhereMaskSet*, ExprList*);
void sqlite3WhereExprAnalyze(SrcList*, WhereClause*);
void sqlite3WhereTabFuncArgs(Parse*, SrcItem*, WhereClause*);







#endif /* !defined(SQLITE_WHEREINT_H) */
