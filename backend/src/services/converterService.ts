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
    console.log(ext, targetFormat);
    try {
      // Route to appropriate service based on file extension
      switch (targetFormat) {
        case '.psd':
          console.log('psd go here');
          return await PSDService.convertPSD(filePath, targetFormat, options.quality);
        
        case 'svg':
          if (ext === '.svg') {
            // Optimize SVG using SVGO
            console.log('svg go here')
            return await SVGService.optimizeSVG(filePath); 
          } else {
            return await SVGService.traceSVG(filePath);
          }
        
        case 'ai':
          console.log('ai goes here');
          return await AIService.convertAI(filePath, targetFormat);
        
        case 'jpg':
        case 'jpeg':
        case 'png':
        case 'bmp':
        case 'webp':
        case 'tiff':
          console.log('pixel go here');
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