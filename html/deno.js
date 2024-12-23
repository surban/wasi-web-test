import * as impl from "./wasi-web-test.js";

async function jsMain() {
    console.info("Start");
    console.info("crossOriginIsolated: ", globalThis.crossOriginIsolated);

    impl.__runtimeLogConfig.filter = "debug";
    const instance = await impl.default({});
    console.log("got WASM instance", instance);

    const stdoutTask = (async () => {
        let stdoutBuffer = '';
        for await (const chunk of instance.stdout) {
            const decoder = new TextDecoder();
            const text = decoder.decode(chunk);
            stdoutBuffer += text;

            let newlineIndex;
            while ((newlineIndex = stdoutBuffer.indexOf('\n')) !== -1) {
                const line = stdoutBuffer.slice(0, newlineIndex + 1);
                stdoutBuffer = stdoutBuffer.slice(newlineIndex + 1);
                console.info("%c==== STDOUT: ", "color: blue;", line.trim());
            }
        }

        // Print any remaining buffered text
        if (stdoutBuffer.length > 0) {
            console.info("%c==== STDOUT: ", "color: blue;", stdoutBuffer.trim());
        }
    })();

    const stderrTask = (async () => {
        let stderrBuffer = '';
        for await (const chunk of instance.stderr) {
            const decoder = new TextDecoder();
            const text = decoder.decode(chunk);
            stderrBuffer += text;

            let newlineIndex;
            while ((newlineIndex = stderrBuffer.indexOf('\n')) !== -1) {
                const line = stderrBuffer.slice(0, newlineIndex + 1);
                stderrBuffer = stderrBuffer.slice(newlineIndex + 1);
                console.info("%c==== STDERR: ", "color: red;", line.trim());
            }
        }

        // Print any remaining buffered text
        if (stderrBuffer.length > 0) {
            console.info("%c==== STDERR: ", "color: red;", stderrBuffer.trim());
        }
    })();

    console.info("calling double 10");
    let res = impl.sync_double(10);
    console.info("result: ", res);

    //await impl.simple_wait_test();

    await impl.simple_thread_test();


    // 
    //     console.info("thread test");
    //     impl.thread_test();

    // await new Promise(resolve => setTimeout(resolve, 1000));

    //impl.mutex_test();

}

await jsMain();
// 
// await new Promise(resolve => setTimeout(resolve, 3000));
// console.info("Slept for 3 seconds");
// 
// await p;
