export const formatAmount = (amount: number) => Number(amount.toFixed(1)).toString()

export class ApiError extends Error {
  constructor(
    public readonly status: number,
    message: string,
  ) {
    super(message)
  }
}
