import path from 'path';
import OpenCVService from './imageService';
import PSDService from './psdService';
import SVGService from './svgService';
import AIService from './aiService'; 

interface ConversionOptions {
    quality?: number;
}

class ConverterService {
  async convertImage(filePath: string, targetFormat: string, options: ConversionOptions = {}) {
    const ext = path.extname(filePath).toLowerCase();
    
    try {
      // Route to appropriate service based on file extension
      switch (ext) {
        case '.psd':
          return await PSDService.convertPSD(filePath, targetFormat, options.quality);
        
        case '.svg':
          if (targetFormat === 'svg') {
            return await SVGService.optimizeSVG(filePath);
          } else {
            // Convert SVG to other formats using OpenCV
            return await OpenCVService.convertImage(filePath, targetFormat, options.quality);
          }
        
        case '.ai':
          return await AIService.convertAI(filePath, targetFormat);
        
        case '.jpg':
        case '.jpeg':
        case '.png':
        case '.bmp':
        case '.webp':
        case '.tiff':
          return await OpenCVService.convertImage(filePath, targetFormat, options.quality);
        
        default:
          throw new Error(`Unsupported file format: ${ext}`);
      }
    } catch (error) {
      console.error('Conversion error:', error);
      throw error;
    }
  }
  
  async extractPSDLayers(filePath:string, targetFormat = 'png') {
    if (path.extname(filePath).toLowerCase() !== '.psd') {
      throw new Error('Can only extract layers from PSD files');
    }
    
    return await PSDService.extractLayers(filePath, targetFormat);
  }
}

export default new ConverterService();