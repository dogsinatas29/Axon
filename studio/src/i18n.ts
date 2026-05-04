/**
 * AXON Studio i18n Dictionary
 */

export const translations: Record<string, any> = {
  ko_KR: {
    boards: "게시판",
    dashboard: "종합 대시보드",
    workBoard: "작업 게시판 (Work Board)",
    office: "인사 관리 (Office)",
    boss: "사장 게시판 (Boss)",
    nogari: "노가리 게시판 (Lounge)",
    signals: "실시간 시그널 (Signals)",
    factoryOverview: "공장 개요",
    activeThreads: "활성 스레드",
    totalSignals: "총 시그널",
    latestStatus: "최근 상태",
    recentStrategicThreads: "최근 전략 스레드",
    viewAll: "전체 보기",
    pauseFactory: "공장 가동 중지",
    resumeFactory: "공장 가동 재개",
    noThreads: "활성 스레드가 없습니다.",
    allSystemsNominal: "모든 시스템 정상 작동 중.",
    silenceInFactory: "공장이 정적에 휩싸였습니다...",
    controlTower: "관제탑",
    boardsHeader: "게시판 목록",
    workBoardTitle: "작업 게시판",
    noWorkThreads: "작업 게시판에 활성 스레드가 없습니다...",
    realTimeSignals: "실시간 공장 시그널"
  },
  en_US: {
    boards: "BOARDS",
    dashboard: "Integrated Dashboard",
    workBoard: "Work Board",
    office: "Office (HR)",
    boss: "Boss Board",
    nogari: "Lounge (Nogari)",
    signals: "Real-time Signals",
    factoryOverview: "Factory Overview",
    activeThreads: "ACTIVE THREADS",
    totalSignals: "TOTAL SIGNALS",
    latestStatus: "LATEST STATUS",
    recentStrategicThreads: "Recent Strategic Threads",
    viewAll: "VIEW ALL",
    pauseFactory: "PAUSE FACTORY",
    resumeFactory: "RESUME FACTORY",
    noThreads: "No threads active.",
    allSystemsNominal: "All systems nominal.",
    silenceInFactory: "Silence in the factory...",
    controlTower: "Control Tower",
    boardsHeader: "BOARDS",
    workBoardTitle: "Work Board",
    noWorkThreads: "No active threads in the Work Board...",
    realTimeSignals: "Real-time Factory Signals"
  },
  ja_JP: {
    boards: "掲示板",
    dashboard: "統合ダッシュボード",
    workBoard: "作業掲示板 (Work Board)",
    office: "人事管理 (Office)",
    boss: "社長掲示板 (Boss)",
    nogari: "ラウンジ (Lounge)",
    signals: "リアルタイムシグナル (Signals)",
    factoryOverview: "工場の概要",
    activeThreads: "アクティブなスレッド",
    totalSignals: "総シ그ナル",
    latestStatus: "最新ステータス",
    recentStrategicThreads: "最近の戦略スレッド",
    viewAll: "すべて表示",
    pauseFactory: "工場停止",
    resumeFactory: "工場再開",
    noThreads: "アクティブなスレッドはありません。",
    allSystemsNominal: "すべてのシステムは正常です。",
    silenceInFactory: "工場は静まり返っています...",
    controlTower: "管制塔",
    boardsHeader: "掲示板リスト",
    workBoardTitle: "作業掲示板",
    noWorkThreads: "作業掲示板에 アクティブなスレッドはありません...",
    realTimeSignals: "リアルタイム工場シグナル"
  }
};

export const getTranslation = (locale: string) => {
  return translations[locale] || translations['en_US'];
};
