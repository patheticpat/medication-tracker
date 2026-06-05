export const formatAmount = (amount: number) => Number(amount.toFixed(1)).toString()

export const formatUnit = (amount: number, unit: string, unitSingular?: string): string =>
  amount === 1 && unitSingular ? unitSingular : unit

export const parseDate = (dateStr: string): Date => {
  const [year, month, day] = dateStr.split('-').map(Number)
  return new Date(year!, month! - 1, day)
}
