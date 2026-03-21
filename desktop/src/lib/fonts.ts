// Google Fonts loader for text logo feature
// Uses the Google Fonts API to dynamically load fonts

const loadedFonts = new Set<string>()

// Modern aesthetic fonts available for text logos
export const TEXT_FONTS = [
  // Modern Sans-Serif
  { name: 'Outfit', family: 'Outfit', weights: [300, 400, 500, 600, 700, 800, 900] },
  { name: 'Poppins', family: 'Poppins', weights: [300, 400, 500, 600, 700, 800, 900] },
  { name: 'Inter', family: 'Inter', weights: [300, 400, 500, 600, 700, 800, 900] },
  { name: 'Space Grotesk', family: 'Space Grotesk', weights: [300, 400, 500, 600, 700] },
  { name: 'DM Sans', family: 'DM Sans', weights: [400, 500, 600, 700] },
  { name: 'Sora', family: 'Sora', weights: [300, 400, 500, 600, 700, 800] },
  { name: 'Manrope', family: 'Manrope', weights: [300, 400, 500, 600, 700, 800] },
  // Bold & Display
  { name: 'Rubik', family: 'Rubik', weights: [300, 400, 500, 600, 700, 800, 900] },
  { name: 'Montserrat', family: 'Montserrat', weights: [300, 400, 500, 600, 700, 800, 900] },
  { name: 'Bebas Neue', family: 'Bebas Neue', weights: [400] },
  { name: 'Oswald', family: 'Oswald', weights: [300, 400, 500, 600, 700] },
  { name: 'Archivo Black', family: 'Archivo Black', weights: [400] },
  // Rounded & Friendly
  { name: 'Nunito', family: 'Nunito', weights: [300, 400, 500, 600, 700, 800, 900] },
  { name: 'Quicksand', family: 'Quicksand', weights: [300, 400, 500, 600, 700] },
  { name: 'Comfortaa', family: 'Comfortaa', weights: [300, 400, 500, 600, 700] },
  { name: 'Varela Round', family: 'Varela Round', weights: [400] },
  // Creative & Stylish
  { name: 'Righteous', family: 'Righteous', weights: [400] },
  { name: 'Bungee', family: 'Bungee', weights: [400] },
  { name: 'Pacifico', family: 'Pacifico', weights: [400] },
  { name: 'Lobster', family: 'Lobster', weights: [400] },
  { name: 'Permanent Marker', family: 'Permanent Marker', weights: [400] },
  // Serif & Elegant
  { name: 'Playfair Display', family: 'Playfair Display', weights: [400, 500, 600, 700, 800, 900] },
  { name: 'Lora', family: 'Lora', weights: [400, 500, 600, 700] },
  { name: 'Merriweather', family: 'Merriweather', weights: [300, 400, 700, 900] },
  // Monospace & Tech
  { name: 'JetBrains Mono', family: 'JetBrains Mono', weights: [300, 400, 500, 600, 700, 800] },
  { name: 'Fira Code', family: 'Fira Code', weights: [300, 400, 500, 600, 700] },
  { name: 'Source Code Pro', family: 'Source Code Pro', weights: [300, 400, 500, 600, 700, 800, 900] },
]

/**
 * Load a single Google Font with specific weights
 */
export function loadGoogleFont(fontFamily: string, weights: number[] = [400, 700]): Promise<void> {
  const fontKey = `${fontFamily}:${weights.join(',')}`
  
  if (loadedFonts.has(fontKey)) {
    return Promise.resolve()
  }

  return new Promise((resolve, reject) => {
    const weightsParam = weights.join(';')
    const fontUrl = `https://fonts.googleapis.com/css2?family=${encodeURIComponent(fontFamily)}:wght@${weightsParam}&display=swap`
    
    // Check if already loaded in document
    const existingLink = document.querySelector(`link[href="${fontUrl}"]`)
    if (existingLink) {
      loadedFonts.add(fontKey)
      resolve()
      return
    }

    const link = document.createElement('link')
    link.rel = 'stylesheet'
    link.href = fontUrl
    
    link.onload = () => {
      loadedFonts.add(fontKey)
      resolve()
    }
    
    link.onerror = () => {
      reject(new Error(`Failed to load font: ${fontFamily}`))
    }
    
    document.head.appendChild(link)
  })
}

/**
 * Preload all text fonts for the font picker
 */
export async function preloadAllTextFonts(): Promise<void> {
  // Load fonts in parallel batches to avoid overwhelming the browser
  const batchSize = 5
  const fonts = TEXT_FONTS
  
  for (let i = 0; i < fonts.length; i += batchSize) {
    const batch = fonts.slice(i, i + batchSize)
    await Promise.all(
      batch.map(font => loadGoogleFont(font.family, font.weights).catch(() => {
        console.warn(`Failed to load font: ${font.family}`)
      }))
    )
  }
}

/**
 * Load a specific font for text rendering.
 * Waits for the font to be actually ready for rendering, not just the CSS stylesheet.
 */
export async function ensureFontLoaded(fontFamily: string): Promise<void> {
  const font = TEXT_FONTS.find(f => f.family === fontFamily)
  if (font) {
    await loadGoogleFont(font.family, font.weights)
  } else {
    // Try loading with default weights
    await loadGoogleFont(fontFamily, [400, 700])
  }
  
  // Wait for the actual font file to be loaded, not just the CSS stylesheet
  // This prevents FOUT (Flash of Unstyled Text) that causes text spacing jumps
  if ('fonts' in document) {
    try {
      // Check if font is already loaded
      const fontCheck = `16px "${fontFamily}"`
      if (!document.fonts.check(fontCheck)) {
        // Wait for font to be ready
        await document.fonts.load(fontCheck)
      }
      // Also wait for all fonts to finish loading
      await document.fonts.ready
    } catch (e) {
      console.warn('Font loading check failed:', e)
    }
  }
}
