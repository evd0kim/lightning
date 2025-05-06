#include "config.h"
#include <ccan/array_size/array_size.h>
#include <ccan/crypto/siphash24/siphash24.h>
#include <ccan/htable/htable_type.h>
#include <ccan/json_escape/json_escape.h>
#include <ccan/json_out/json_out.h>
#include <ccan/tal/str/str.h>
#include <common/daemon.h>
#include <common/gossmap.h>
#include <common/gossmods_listpeerchannels.h>
#include <common/json_param.h>
#include <common/json_stream.h>
#include <common/memleak.h>
#include <common/pseudorand.h>
#include <common/route.h>
#include <common/wireaddr.h>
#include <errno.h>
#include <plugins/libplugin.h>
#include <stdarg.h>

struct randpay {
	struct pubkey local_id;
	/* Access via get_gossmap() */
	struct gossmap *global_gossmap;
	/* Are we to wait for all parts to complete before returning? */
	bool slow_mode;
};

static struct randpay *randpay_of(struct plugin *plugin)
{
	return plugin_get_data(plugin, struct randpay);
}

static struct gossmap *get_gossmap(struct randpay *randpay)
{
	gossmap_refresh(randpay->global_gossmap);
	return randpay->global_gossmap;
}

static struct command_result *json_randpay_core(struct command *cmd,
						const char *buffer,
						const jsmntok_t *params,
						bool as_pay)
{
	struct randpay *randpay = randpay_of(cmd->plugin);
	struct node_id *destination = NULL;

	//plugin_err(cmd->plugin, "Some error");
	if (!param_check(cmd, buffer, params,
			 p_opt("destination", param_node_id, &destination),
			 NULL))
		return command_param_failed();

	if (!randpay->global_gossmap) {
		plugin_log(cmd->plugin, LOG_DBG, "gossip data in not ready yet");
		return notification_handled(cmd);
	}

	if (destination) {
		struct gossmap *gossmap = get_gossmap(randpay);
		plugin_log(cmd->plugin, LOG_DBG, "looking for % in gossip data", destination);
		struct gossmap_node *dst;
		dst = gossmap_find_node(gossmap, destination);
		if (dst) {
			struct json_stream *response;
			response = jsonrpc_stream_success(cmd);
			json_object_start(response, "replace");
			json_add_string(response, "jsonrpc", "2.0");
			json_add_string(response, "method", "keysend");
			json_object_start(response, "params");
			json_add_node_id(response, "destination", destination);
			json_add_string(response, "amount_msat", "1000000"); /* Default 1000 sats */
			json_object_end(response);
			json_object_end(response);
			return command_finished(cmd, response);
		} else {
			return command_fail(cmd, JSONRPC2_INVALID_PARAMS,
					    "Destination must a public node");
		}

	}

	return command_fail(cmd, JSONRPC2_INVALID_PARAMS, "Random node flow is not implemented");
}

static struct command_result *json_randpay(struct command *cmd,
					const char *buffer,
					const jsmntok_t *params)
{
	return json_randpay_core(cmd, buffer, params, false);
}

static const char *init(struct command *init_cmd,
			const char *buf UNUSED, const jsmntok_t *config UNUSED)
{
	struct plugin *p = init_cmd->plugin;
	struct randpay *randpay = randpay_of(p);
	plugin_log(p, LOG_DBG, "loading gossip data");
	randpay->global_gossmap = gossmap_load(randpay,
					    GOSSIP_STORE_FILENAME,
					    plugin_gossmap_logcb,
					    p);
	if (!randpay->global_gossmap)
		plugin_err(p, "Could not load gossmap %s: %s",
			   GOSSIP_STORE_FILENAME, strerror(errno));
	plugin_log(p, LOG_DBG, "gossip data loaded");
	return NULL;
}

static const struct plugin_command commands[] = {
	{
		"randpay",
		json_randpay,
	},
};

int main(int argc, char *argv[])
{
	struct randpay *randpay;

	setup_locale();
	randpay = tal(NULL, struct randpay);
	randpay->slow_mode = false;
	plugin_main(argv, init, take(randpay),
		    PLUGIN_RESTARTABLE, true, NULL,
		    commands, ARRAY_SIZE(commands),
		    /* notifications, ARRAY_SIZE(notifications), */ NULL, 0,
		    /* hooks, ARRAY_SIZE(hooks) */ NULL, 0,
	            NULL, 0,
		    plugin_option_dynamic("randpay-slow-mode", "bool",
					  "Wait until all parts have completed before returning success or failure",
					  bool_option, bool_jsonfmt, &randpay->slow_mode),
		    NULL);
}
