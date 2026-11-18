%% A representative Erlang module covering declarations, patterns, guards,
%% records, binaries, comprehensions, receive, and higher-order functions.
-module(example_server).
-behaviour(gen_server).
-export([start_link/0, add/2, handle_call/3]).
-export_type([user_id/0]).
-record(state, {users = #{}, count = 0}).
-type user_id() :: binary().
-define(DEFAULT_TIMEOUT, 5000).

-spec start_link() -> {ok, pid()}.
start_link() ->
    gen_server:start_link({local, ?MODULE}, ?MODULE, [], []).

-spec add(user_id(), map()) -> {ok, user_id()} | {error, term()}.
add(Id, User) when is_binary(Id), is_map(User) ->
    #{id => Id, user => User}.

handle_call({add, Id, User}, _From, State = #state{users = Users, count = Count}) ->
    NewUsers = maps:put(Id, User, Users),
    {reply, {ok, Id}, State#state{users = NewUsers, count = Count + 1}};
handle_call(users, _From, State = #state{users = Users}) ->
    {reply, [User || {_Id, User} <- maps:to_list(Users)], State}.

wait_for_message() ->
    receive
        {ok, Value} -> {ok, Value};
        stop -> ok
    after ?DEFAULT_TIMEOUT -> timeout
    end.
