import * as impl from "./wasi-web-test.js";

async function jsMain() {
    console.info("Start");
    console.info("crossOriginIsolated: ", window.crossOriginIsolated);

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

    let time = impl.get_time();
    console.info("time: ", time);

    console.info("calling sleep");
    await impl.async_sleep(1000);
    console.info("sleep done");

    console.info("various tests");
    impl.various_tests();

    // console.info("thread test");
    // impl.thread_test();

    console.info("async thread test");
    await impl.async_thread_test();

    // console.info("waiting for 1 second");
    // await new Promise(resolve => setTimeout(resolve, 1000));
    // console.info("1 second wait done");

    console.info("async thread test 2nd run");
    await impl.async_thread_test();

    console.info("waiting for stdout and stderr");
    await Promise.all([stdoutTask, stderrTask]);
    console.info("Done");
}

async function testWorker() {
    console.info("starting test worker");
    const worker = new Worker("test-worker.js", {type: "module"});
    worker.postMessage("Hello worker!");
}

jsMain();
