import svgo from 'svgo';
import fs from 'fs/promises';
import potrace from 'potrace';
import path from 'path';

class SVGService {
    async traceSVG(inputPath: string) {
        const outputPath = this.generateOutputPath(inputPath);
        const trace = potrace.trace;
        const params = {
            turdsize: 100,
            optcurve: true,
            alphamax: 1,
            opttolerance: 0.2,
            color: 'black',
            background: 'white',
            format: 'svg'
        };

        return new Promise((resolve, reject) => {
            trace(inputPath, params, (err, svg) => {
                if (err) {
                    reject(err);
                } else {

                    fs.writeFile(outputPath, svg, 'utf8')
                    .then(() => this.optimizeSVG(outputPath))
                    .then(resolve)
                    .catch(reject);
                }
            });
        });
    }
    
    async optimizeSVG(inputPath: string) {
        const svgBuffer = await fs.readFile(inputPath);
    
        // Check if the file starts with valid UTF-8 text or XML declaration
        let svgString;
        if (svgBuffer.toString('utf8', 0, 5).match(/^(<\?xml|<svg)/)) {
            svgString = svgBuffer.toString('utf8');
        } else {
            // Try to detect encoding or remove BOMs
            svgString = svgBuffer.toString('utf8').replace(/^\uFEFF/, ''); // Remove BOM if present
        }

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
            console.log('Optimized SVG:', outputData);
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