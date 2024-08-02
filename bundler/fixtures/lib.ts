export async function execute(name: string): Promise<string> {
    console.log('Executing lib');
    return `Hello ${name}!`;
}

// 此函数不会被使用到，会在bundle之后就没了
function not_used() {
    console.log('This function is not used')
}
