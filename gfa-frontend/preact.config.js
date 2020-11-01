export default (config, env, helpers) => {
    const { plugin } = helpers.getPluginsByName(config, 'DefinePlugin')[0];
    Object.assign(plugin.definitions, {
        API_URL: JSON.stringify(process.env.API_URL),
    });
}
