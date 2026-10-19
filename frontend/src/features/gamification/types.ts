export interface UserStats {
  materialsSubmitted: number
  transfersCount: number
  totalEarned: bigint
  joinedAt?: number
}

export interface Badge {
  id: string
  name: string
  description: string
  icon: string
  earned: boolean
}

export interface Level {
  level: number
  title: string
  xp: number
  xpForNext: number
  progressPct: number
}

export interface Challenge {
  id: string
  title: string
  description: string
  target: number
  current: number
  progressPct: number
  completed: boolean
  type: 'daily' | 'weekly'
  expiresAt: number
}

export interface LeaderboardEntry {
  rank: number
  address: string
  displayName: string
  xp: number
  level: number
}
