import svgo from 'svgo';
import fs from 'fs/promises';
import path from 'path';

class SVGService {
    async optimizeSVG(inputPath: string) {
        const svgString = await fs.readFile(inputPath, 'utf8');
        const outputPath = this.generateOutputPath(inputPath);

        interface SVGOSuccess { data: string; [key: string]: any; }
        interface SVGOError { error: string; [key: string]: any; }
        type SVGOResult = SVGOSuccess | SVGOError;

        const result = svgo.optimize(svgString, {
            plugins: [
              'removeDoctype',
              'removeXMLProcInst',
              'removeComments',
              'removeMetadata',
              'removeEditorsNSData',
              'cleanupAttrs',
              'minifyStyles',
              'convertStyleToAttrs',
              'cleanupIds',
              'removeUselessDefs',
              'cleanupNumericValues',
              'convertColors',
              'removeUnknownsAndDefaults',
              'removeNonInheritableGroupAttrs',
              'removeUselessStrokeAndFill',
              'removeViewBox',
              'cleanupEnableBackground',
              'removeHiddenElems',
              'removeEmptyText',
              'convertShapeToPath',
              'convertEllipseToCircle',
              'moveElemsAttrsToGroup',
              'moveGroupAttrsToElems',
              'collapseGroups',
              'convertPathData',
              'convertTransform',
              'removeEmptyAttrs',
              'removeEmptyContainers',
              'mergePaths',
              'removeUnusedNS',
              'sortDefsChildren',
              'removeTitle',
              'removeDesc'
            ]
        }) as SVGOResult;
        
        if ('data' in result) {
            await fs.writeFile(outputPath, result.data);
            return outputPath;
        } else {
            throw new Error(`SVG optimization failed: ${result.error}`);
        }
        
    }

    generateOutputPath(inputPath: string) {
        const dirName = path.dirname(inputPath);
        const baseName = path.basename(inputPath, path.extname(inputPath));
        return path.join(dirName, `${baseName}-optimized.svg`);
    }
}

export default new SVGService();