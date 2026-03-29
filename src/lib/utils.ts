import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/**
 * 将十六进制颜色转换为 RGBA 格式
 * @param hex - 十六进制颜色（如 "#426666"）
 * @param alpha - 透明度（0-1）
 * @returns RGBA 颜色字符串（如 "rgba(66, 102, 102, 0.75)"）
 */
export function hexToRgba(hex: string, alpha: number): string {
  const cleanHex = hex.replace('#', '')
  const r = parseInt(cleanHex.slice(0, 2), 16)
  const g = parseInt(cleanHex.slice(2, 4), 16)
  const b = parseInt(cleanHex.slice(4, 6), 16)
  const clampedAlpha = Math.max(0, Math.min(1, alpha))

  return `rgba(${r}, ${g}, ${b}, ${clampedAlpha})`
}
