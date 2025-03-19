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
                {
                    name: 'preset-default',
                    params: {
                        overrides: {
                            removeXMLProcInst: false,
                            removeDoctype: false,
                            removeViewBox: false,
                            inlineStyles: false,
                            removeMetadata: false,
                            removeTitle: false
                        }
                    }
                },
                'removeComments',
                'removeUselessDefs',
                'cleanupNumericValues',
                'mergePaths',
                'sortAttrs'
            ]
        }) as SVGOResult;
        
        if ('data' in result) {
            // Add XML declaration if missing
            let outputData = result.data;
            if (!outputData.includes('<?xml')) {
                outputData = '<?xml version="1.0" encoding="UTF-8" standalone="no"?>\n' + outputData;
            }
            
            if (!outputData.includes('<style>') && !outputData.includes('style="')) {
                const style = `
                    <style>
                        svg {
                            display: block;
                            width: 100%;
                            height: auto;
                        }
                    </style>
                `;
                outputData = outputData.replace(/(<svg[^>]*>)/, `$1\n${style}`);
            }

            await fs.writeFile(outputPath, outputData);
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