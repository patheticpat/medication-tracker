export interface Medication {
  id: string
  name: string
  unit: string
  schedule: Schedule
  warningThreshold: number
  snoozed: boolean
  logs?: LogEntry[]
}

export interface MedicationWithStats extends Medication {
  stock: number
  daysRemaining: number
}

export interface CreateMedication extends Omit<Medication, 'id' | 'logs' | 'snoozed'> {
  initialStock: number
}

export type UpdateMedication = Partial<Omit<Medication, 'id' | 'logs'>>
export type UpdateMedicationArgs = { id: string } & UpdateMedication

export type LogEntryKind = 'baseline' | 'refill'

export interface LogEntry {
  kind: LogEntryKind
  id: string
  date: string
  amount: number
  note?: string
}

export type CreateLogEntry = Omit<LogEntry, 'date' | 'id'>

export type Schedule =
  | { kind: 'daily'; amount: number }
  | { kind: 'weekly'; amount: number; dayOfWeek: number }
